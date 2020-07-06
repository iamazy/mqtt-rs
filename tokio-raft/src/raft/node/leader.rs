use std::collections::HashMap;
use crate::raft::node::{RoleNode, Node, Status, HEARTBEAT_INTERVAL};
use crate::raft::node::follower::Follower;
use crate::error::{Result, Error};
use crate::raft::state::Instruction;
use crate::raft::message::{Address, Event, Message, Request, Response};
use ::log::{debug, info, warn};

/// A leader serves requests and replicates the log to followers
#[derive(Debug)]
pub struct Leader {
    /// Number of ticks since last heartbeat
    heartbeat_ticks: u64,
    /// The next index to replicate to peer
    peer_next_index: HashMap<String, u64>,
    /// The last index known to be replicated on a peer
    peer_last_index: HashMap<String ,u64>
}

impl Leader {
    /// Creates a new leader role
    pub fn new(peers: Vec<String>, last_index: u64) -> Self {
        let mut leader = Self {
            heartbeat_ticks: 0,
            peer_next_index: HashMap::new(),
            peer_last_index: HashMap::new()
        };
        for peer in peers {
            leader.peer_next_index.insert(peer.clone(), last_index + 1);
            leader.peer_last_index.insert(peer.clone(), 0);
        }
        leader
    }
}

impl RoleNode<Leader> {

    /// Transforms the leader into a follower
    fn become_follower(mut self, term: u64, leader: &str) -> Result<RoleNode<Follower>> {
        info!("Discovered new leader {} for term {}, following", leader, term);
        self.term = term;
        self.log.save_term(term, None)?;
        self.state_tx.send(Instruction::Abort)?;
        self.become_role(Follower::new(Some(leader), None))
    }

    pub fn append(&mut self, command: Option<Vec<u8>>) -> Result<u64> {
        let entry = self.log.append(self.term, command)?;
        for peer in self.peers.iter() {
            self.replicate(peer)?;
        }
        Ok(entry.index)
    }

    fn commit(&mut self) -> Result<u64> {
        let mut last_indexes = vec![self.log.last_index];
        last_indexes.extend(self.role.peer_last_index.values());
        last_indexes.sort();
        last_indexes.reverse();
        let quorum_index = last_indexes[self.quorum() as usize - 1];

        if quorum_index > self.log.commit_index {
            if let Some(entry) = self.log.get(quorum_index)? {
                if entry.term == self.term {
                    let old_commit_index = self.log.commit_index;
                    self.log.commit(quorum_index)?;
                    let mut scan = self.log.scan((old_commit_index + 1)..=self.log.commit_index);
                    while let Some(entry) = scan.next().transpose()? {
                        self.state_tx.send(Instruction::Apply { entry})?;
                    }
                }
            }
        }
        Ok(self.log.commit_index)
    }

    /// Replicates the log to a peer
    fn replicate(&self, peer: &str) -> Result<()> {
        let peer_next = self.role.peer_next_index
            .get(peer)
            .cloned()
            .ok_or_else(||Error::Internal(format!("Unknown peer {}", peer)))?;
        let base_index = if peer_next > 0 {
            peer_next - 1
        } else {
            0
        };
        let base_term = match self.log.get(base_index)? {
            Some(base) => base.term,
            None if base_index == 0 => 0,
            None => return Err(Error::Internal(format!("Missing base entry {}", base_index))),
        };
        let entries = self.log.scan(peer_next..).collect::<Result<Vec<_>>>()?;
        debug!("Replicating {} entries at base {} to {}", entries.len(), base_index, peer);
        self.send(Address::Peer(peer.to_string()), Event::ReplicateEntries {base_index, base_term, entries})?;
        Ok(())
    }

    pub fn step(mut self, msg: Message) -> Result<Node> {
        if let Err(err) = self.validate(&msg) {
            warn!("Ignoring invalid message: {}", err);
            return Ok(self.into());
        }
        if msg.term > self.term {
            if let Address::Peer(from) = &msg.from {
                return self.become_follower(msg.term, from)?.step(msg);
            }
        }
        match msg.event {
            Event::ConfirmLeader { commit_index, has_committed} => {
                if let Address::Peer(from) = msg.from.clone() {
                    self.state_tx.send(Instruction::Vote {
                        term: msg.term,
                        index: commit_index,
                        address: msg.from
                    })?;
                    if !has_committed {
                        self.replicate(&from)?;
                    }
                }
            }

            Event::AcceptEntries { last_index} => {
                if let Address::Peer(from) = msg.from {
                    self.role.peer_last_index.insert(from.clone(), last_index);
                    self.role.peer_next_index.insert(from, last_index + 1);
                }
                self.commit()?;
            }

            Event::RejectEntries => {
                if let Address::Peer(from) = msg.from {
                    self.role.peer_next_index.entry(from.clone()).and_modify(|i| {
                        if *i > 1{
                            *i -= 1
                        }
                    });
                    self.replicate(&from);
                }
            }

            Event::ClientRequest { id, request: Request::Query(command) } => {
                self.state_tx.send(Instruction::Query {
                    id,
                    address: msg.from,
                    command,
                    term: self.term,
                    index: self.log.commit_index,
                    quorum: self.quorum(),
                })?;
                self.state_tx.send(Instruction::Vote {
                    term: self.term,
                    index: self.log.commit_index,
                    address: Address::Local,
                })?;
                if !self.peers.is_empty() {
                    self.send(
                        Address::Peers,
                        Event::Heartbeat {
                            commit_index: self.log.commit_index,
                            commit_term: self.log.commit_term,
                        },
                    )?;
                }
            }

            Event::ClientRequest { id, request: Request::Mutate(command) } => {
                let index = self.append(Some(command))?;
                self.state_tx.send(Instruction::Notify { id, address: msg.from, index })?;
                if self.peers.is_empty() {
                    self.commit()?;
                }
            }

            Event::ClientRequest { id, request: Request::Status } => {
                let mut status = Box::new(Status {
                    server: self.id.clone(),
                    leader: self.id.clone(),
                    term: self.term,
                    node_last_index: self.role.peer_last_index.clone(),
                    commit_index: self.log.commit_index,
                    apply_index: 0,
                    storage: self.log.store.to_string(),
                    storage_size: self.log.store.size(),
                });
                status.node_last_index.insert(self.id.clone(), self.log.last_index);
                self.state_tx.send(Instruction::Status { id, address: msg.from, status })?
            }

            Event::ClientResponse { id, mut response } => {
                if let Ok(Response::Status(ref mut status)) = response {
                    status.server = self.id.clone();
                }
                self.send(Address::Client, Event::ClientResponse { id, response })?;
            }

            // We ignore these messages, since they are typically additional votes from the previous
            // election that we won after a quorum.
            Event::SolicitVote { .. } | Event::GrantVote => {}

            Event::Heartbeat { .. } | Event::ReplicateEntries { .. } => {
                warn!("Received unexpected message {:?}", msg)
            }
        }
        Ok(self.into())
    }

    /// Processes a logical clock tick.
    pub fn tick(mut self) -> Result<Node> {
        if !self.peers.is_empty() {
            self.role.heartbeat_ticks += 1;
            if self.role.heartbeat_ticks >= HEARTBEAT_INTERVAL {
                self.role.heartbeat_ticks = 0;
                self.send(
                    Address::Peers,
                    Event::Heartbeat {
                        commit_index: self.log.commit_index,
                        commit_term: self.log.commit_term,
                    },
                )?;
            }
        }
        Ok(self.into())
    }
}