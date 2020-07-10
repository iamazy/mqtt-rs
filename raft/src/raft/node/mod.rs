mod leader;
mod follower;
mod candidate;

use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::raft::log::Log;
use crate::raft::node::candidate::Candidate;
use crate::raft::node::follower::Follower;
use crate::raft::node::leader::Leader;
use crate::raft::message::{Message, Address, Event};
use crate::error::{Result, Error};
use crate::raft::state::{State, Driver, Instruction};
use ::log::{debug, info};
use serde_derive::{Deserialize, Serialize};


/// The interval between leader heartbeats, in ticks
const HEARTBEAT_INTERVAL: u64 = 1;

/// The minimum election timeout, in ticks
const ELECTION_TIMEOUT_MIN: u64 = 8 * HEARTBEAT_INTERVAL;

/// The maximum election timeout, in ticks
const ELECTION_TIMEOUT_MAX: u64 = 15 * HEARTBEAT_INTERVAL;

/// Node status
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub server: String,
    pub leader: String,
    pub term: u64,
    pub node_last_index: HashMap<String, u64>,
    pub commit_index: u64,
    pub apply_index: u64,
    pub storage: String,
    pub storage_size: u64,
}

/// The local Raft node state machine
pub enum Node {
    Candidate(RoleNode<Candidate>),
    Follower(RoleNode<Follower>),
    Leader(RoleNode<Leader>)
}

impl Node {
    /// Creates a new Raft node, starting as a follower, or leader if no peers
    pub async fn new(
        id: &str,
        peers: Vec<String>,
        log: Log,
        mut state: Box<dyn State>,
        node_tx: mpsc::UnboundedSender<Message>
    ) -> Result<Self> {
        let applied_index = state.applied_index();
        if applied_index > log.commit_index {
            return Err(Error::Internal(format!(
                "State machine applied index {} greater than log committed index {}",
                applied_index, log.commit_index
            )));
        }
        let (state_tx, state_rx) = mpsc::unbounded_channel();
        let mut driver = Driver::new(state_rx, node_tx.clone());
        if log.commit_index > applied_index {
            info!("Replaying log entries {} to {}", applied_index + 1, log.commit_index);
            driver.replay(&mut *state, log.scan((applied_index + 1)..=log.commit_index))?;
        }
        tokio::spawn(driver.drive(state));

        let (term, voted_for) = log.load_term()?;
        let node = RoleNode {
            id: id.to_owned(),
            peers,
            term,
            log,
            node_tx,
            state_tx,
            queued_reqs: Vec::new(),
            proxied_reqs: HashMap::new(),
            role: Follower::new(None, voted_for.as_deref())
        };
        if node.peers.is_empty() {
            info!("No peers specified, starting as leader");
            let last_index = node.log.last_index;
            Ok(node.become_role(Leader::new(vec![], last_index))?.into())
        } else {
            Ok(node.into())
        }
    }

    /// Returns the node id
    pub fn id(&self) -> String {
        match self {
            Node::Candidate(n) => n.id.clone(),
            Node::Follower(n) => n.id.clone(),
            Node::Leader(n) => n.id.clone()
        }
    }

    pub fn step(self, msg: Message) -> Result<Self> {
        debug!("Steppinng {:?}", msg);
        match self {
            Node::Candidate(n) => n.step(msg),
            Node::Follower(n) => n.step(msg),
            Node::Leader(n) => n.step(msg)
        }
    }

    /// Moves time forward by a tick
    pub fn tick(self) -> Result<Self>{
        match self {
            Node::Candidate(n) => n.tick(),
            Node::Follower(n) => n.tick(),
            Node::Leader(n) => n.tick()
        }
    }
}

impl From<RoleNode<Candidate>> for Node {
    fn from(rn: RoleNode<Candidate>) -> Self {
        Node::Candidate(rn)
    }
}

impl From<RoleNode<Follower>> for Node {
    fn from(rn: RoleNode<Follower>) -> Self {
        Node::Follower(rn)
    }
}

impl From<RoleNode<Leader>> for Node {
    fn from(rn: RoleNode<Leader>) -> Self {
        Node::Leader(rn)
    }
}

/// A Raft node with role R
pub struct RoleNode<R> {
    id: String,
    peers: Vec<String>,
    term: u64,
    log: Log,
    node_tx: mpsc::UnboundedSender<Message>,
    state_tx: mpsc::UnboundedSender<Instruction>,
    /// Keeps track of queued client requests received e.g. during elections
    queued_reqs: Vec<(Address, Event)>,
    /// Keeps track of proxied client request, to abort on new leader election
    proxied_reqs: HashMap<Vec<u8>, Address>,
    role: R
}

impl<R> RoleNode<R> {
    /// Transforms the node into another role
    fn become_role<T>(self, role: T) -> Result<RoleNode<T>> {
        Ok(RoleNode {
            id: self.id,
            peers: self.peers,
            term: self.term,
            log: self.log,
            node_tx: self.node_tx,
            state_tx: self.state_tx,
            queued_reqs: self.queued_reqs,
            proxied_reqs: self.proxied_reqs,
            role
        })
    }

    /// Aborts any proxied requests
    fn abort_proxied(&mut self) -> Result<()> {
        for (id, address) in std::mem::replace(&mut self.proxied_reqs, HashMap::new()) {
            self.send(address, Event::ClientResponse {
                id,
                response: Err(Error::Abort)
            })?;
        }
        Ok(())
    }

    fn forward_queued(&mut self, leader: Address) -> Result<()> {
        for (from, event) in std::mem::replace(&mut self.queued_reqs, Vec::new()) {
            if let Event::ClientRequest { id, .. } = &event {
                self.proxied_reqs.insert(id.clone(), from.clone());
                self.node_tx.send(Message {
                    from: match from {
                        Address::Client => Address::Local,
                        address => address
                    },
                    to: leader.clone(),
                    term: 0,
                    event
                })?;
            }
        }
        Ok(())
    }

    /// Returns the quorum size of the cluster
    fn quorum(&self) -> u64 {
        (self.peers.len() as u64 + 1) / 2 + 1
    }

    fn send(&self, to: Address, event: Event) -> Result<()> {
        let msg = Message {
            term: self.term,
            from: Address::Local,
            to,
            event
        };
        debug!("Sending {:?}", msg);
        Ok(self.node_tx.send(msg)?)
    }

    fn validate(&self, msg: &Message) -> Result<()> {
        match msg.from {
            Address::Peers => return Err(Error::Internal("Message from broadcast address".into())),
            Address::Local => return Err(Error::Internal("Message from local node".into())),
            Address::Client if !matches!(msg.event, Event::ClientRequest{..}) => {
                return Err(Error::Internal("Non-request message from client".into()));
            }
            _ => {}
        }

        // Allowing requests and responses from past terms is fine, since they don't rely on it
        if msg.term < self.term
            && !matches!(msg.event, Event::ClientRequest{..} | Event::ClientResponse{..}){
            return Err(Error::Internal(format!("Message from past term {}", msg.term)));
        }

        match &msg.to {
            Address::Peer(id) if id == &self.id => Ok(()),
            Address::Local => Ok(()),
            Address::Peers => Ok(()),
            Address::Peer(id) => {
                Err(Error::Internal(format!("Received message for other node {}", id)))
            }
            Address::Client => Err(Error::Internal("Received message for client".into()))
        }
    }
}