use rand::Rng;
use crate::error::Result;
use crate::raft::node::{ELECTION_TIMEOUT_MIN, ELECTION_TIMEOUT_MAX, RoleNode, Node};
use crate::raft::node::candidate::Candidate;
use crate::raft::message::{Address, Event, Message, Response};
use crate::raft::state::Instruction;
use ::log::{debug, info, warn};

/// A follower replicates state from a leader
#[derive(Debug)]
pub struct Follower {
    /// The leader, or None if just initialized
    leader: Option<String>,
    /// The number of ticks since the last message from the leader
    leader_seen_ticks: u64,
    /// The timeout before triggering an election
    leader_seen_timeout: u64,
    /// The node we voted for in the current term, if any
    voted_for: Option<String>,
}


impl Follower {
    pub fn new(leader: Option<&str>, voted_for: Option<&str>) -> Self {
        Self {
            leader: leader.map(String::from),
            voted_for: voted_for.map(String::from),
            leader_seen_ticks: 0,
            leader_seen_timeout: rand::thread_rng()
                .gen_range(ELECTION_TIMEOUT_MIN, ELECTION_TIMEOUT_MAX),
        }
    }
}

impl RoleNode<Follower> {
    /// Transforms the node into a candidate
    fn become_candidate(self) -> Result<RoleNode<Candidate>> {
        info!("Starting election for term {}", self.term + 1);
        let mut node = self.become_role(Candidate::new())?;
        node.term += 1;
        node.log.save_term(node.term, None)?;
        node.send(Address::Peers,
                  Event::SolicitVote { last_index: node.log.last_index, last_term: node.log.last_term })?;

        Ok(node)
    }

    fn become_follower(mut self, leader: &str, term: u64) -> Result<RoleNode<Follower>> {
        let mut voted_for = None;
        if term > self.term {
            info!("Discovered new term {}, following leader {}", term, leader);
            self.term = term;
            self.log.save_term(term, None)?;
        } else {
            info!("Discovered leader {}, following", leader);
            voted_for = self.role.voted_for;
        };
        self.role = Follower::new(Some(leader), voted_for.as_deref());
        self.abort_proxied()?;
        self.forward_queued(Address::Peer(leader.to_string()))?;
        Ok(self.into())
    }

    fn is_leader(&self, from: &Address) -> bool {
        match (&self.role.leader, from) {
            (Some(leader), Address::Peer(from)) if leader == from => true,
            _ => false
        }
    }

    pub fn step(mut self, msg: Message) -> Result<Node> {
        if let Err(err) = self.validate(&msg) {
            warn!("Ignoring invalid message: {}", err);
            return Ok(self.into());
        }
        if let Address::Peer(from) = &msg.from {
            if msg.term > self.term || self.role.leader.is_none() {
                return self.become_follower(from, msg.term)?.step(msg);
            }
        }
        if self.is_leader(&msg.from) {
            self.role.leader_seen_ticks = 0;
        }

        match msg.event {
            Event::Heartbeat { commit_index, commit_term } => {
                if self.is_leader(&msg.from) {
                    let has_committed = self.log.has(commit_index, commit_term)?;
                    if has_committed && commit_index > self.log.commit_index {
                        let old_commit_index = self.log.commit_index;
                        self.log.commit(commit_index)?;
                        let mut scan = self.log.scan((old_commit_index + 1)..=commit_index);
                        while let Some(entry) = scan.next().transpose()? {
                            self.state_tx.send(Instruction::Apply { entry})?;
                        }
                    }
                    self.send(msg.from, Event::ConfirmLeader { commit_index, has_committed})?;
                }
            }
            Event::SolicitVote { last_index, last_term} => {
                if let Some(voted_for) = &self.role.voted_for {
                    if msg.from != Address::Peer(voted_for.clone()) {
                        return Ok(self.into());
                    }
                }
                if last_term < self.log.last_term {
                    return Ok(self.into());
                }
                if last_term == self.log.last_term && last_index < self.log.last_index {
                    return Ok(self.into());
                }
                if let Address::Peer(from) = msg.from {
                    info!("Voting for {} in term {} election", from, self.term);
                    self.send(Address::Peer(from.clone()), Event::GrantVote)?;
                    self.log.save_term(self.term, Some(&from))?;
                    self.role.voted_for = Some(from);
                }
            }
            Event::ReplicateEntries { base_index, base_term, entries} => {
                if self.is_leader(&msg.from) {
                    if base_index > 0 && !self.log.has(base_index, base_term)? {
                        debug!("Rejecting log entries at base {}", base_index);
                        self.send(msg.from, Event::RejectEntries)?;
                    } else {
                        let last_index = self.log.splice(entries)?;
                        self.send(msg.from, Event::AcceptEntries { last_index})?;
                    }
                }
            }
            Event::ClientRequest { ref id, .. } => {
                if let Some(leader) = self.role.leader.as_deref() {
                    self.proxied_reqs.insert(id.clone(), msg.from);
                    self.send(Address::Peer(leader.to_string()), msg.event)?;
                } else {
                    self.queued_reqs.push((msg.from, msg.event));
                }
            }

            Event::ClientResponse { id, mut response} => {
                if let Ok(Response::Status(ref mut status)) = response {
                    status.server = self.id.clone();
                }
                self.proxied_reqs.remove(&id);
                self.send(Address::Client, Event::ClientResponse { id, response})?;
            }

            Event::GrantVote => {},
            Event::ConfirmLeader { ..} | Event::AcceptEntries { ..} | Event::RejectEntries {..} => {
                warn!("Received unexpected message {:?}", msg);
            }
        };
        Ok(self.into())
    }

    pub fn tick(mut self) -> Result<Node> {
        self.role.leader_seen_ticks += 1;
        if self.role.leader_seen_ticks >= self.role.leader_seen_timeout {
            Ok(self.become_candidate()?.into())
        } else {
            Ok(self.into())
        }
    }
}