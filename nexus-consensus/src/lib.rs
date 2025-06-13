
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod raft;
pub mod log;
pub mod snapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Uuid,
    pub address: String,
    pub is_leader: bool,
}

pub trait ConsensusEngine {
    async fn propose(&mut self, data: Vec<u8>) -> anyhow::Result<u64>;
    async fn get_leader(&self) -> Option<Uuid>;
    async fn is_leader(&self) -> bool;
}

#[derive(Debug)]
pub struct RaftConsensus {
    node_id: Uuid,
    nodes: HashMap<Uuid, Node>,
    current_term: u64,
    voted_for: Option<Uuid>,
    log: Vec<LogEntry>,
    commit_index: u64,
    last_applied: u64,
    is_leader: bool,
}

impl RaftConsensus {
    pub fn new(node_id: Uuid) -> Self {
        Self {
            node_id,
            nodes: HashMap::new(),
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            is_leader: false,
        }
    }
    
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }
}

impl ConsensusEngine for RaftConsensus {
    async fn propose(&mut self, data: Vec<u8>) -> anyhow::Result<u64> {
        if !self.is_leader {
            return Err(anyhow::anyhow!("Not a leader"));
        }
        
        let entry = LogEntry {
            term: self.current_term,
            index: self.log.len() as u64,
            command: data,
        };
        
        self.log.push(entry);
        Ok(self.log.len() as u64 - 1)
    }
    
    async fn get_leader(&self) -> Option<Uuid> {
        self.nodes.values()
            .find(|node| node.is_leader)
            .map(|node| node.id)
    }
    
    async fn is_leader(&self) -> bool {
        self.is_leader
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_creation() {
        let node_id = Uuid::new_v4();
        let consensus = RaftConsensus::new(node_id);
        assert_eq!(consensus.node_id, node_id);
        assert!(!consensus.is_leader().await);
    }
}
