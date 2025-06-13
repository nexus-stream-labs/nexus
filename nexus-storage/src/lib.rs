
use rocksdb::{DB, Options};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod log;
pub mod index;
pub mod compaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub offset: u64,
    pub timestamp: u64,
    pub key: Option<String>,
    pub value: Vec<u8>,
}

pub trait StorageEngine {
    async fn append(&mut self, entry: LogEntry) -> anyhow::Result<u64>;
    async fn read(&self, offset: u64) -> anyhow::Result<Option<LogEntry>>;
    async fn read_range(&self, start_offset: u64, end_offset: u64) -> anyhow::Result<Vec<LogEntry>>;
    async fn compact(&mut self) -> anyhow::Result<()>;
}

pub struct RocksDbStorage {
    db: DB,
}

impl RocksDbStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        
        Ok(Self { db })
    }
}

impl StorageEngine for RocksDbStorage {
    async fn append(&mut self, entry: LogEntry) -> anyhow::Result<u64> {
        let key = entry.offset.to_be_bytes();
        let value = bincode::serialize(&entry)?;
        self.db.put(key, value)?;
        Ok(entry.offset)
    }
    
    async fn read(&self, offset: u64) -> anyhow::Result<Option<LogEntry>> {
        let key = offset.to_be_bytes();
        match self.db.get(key)? {
            Some(value) => {
                let entry: LogEntry = bincode::deserialize(&value)?;
                Ok(Some(entry))
            },
            None => Ok(None),
        }
    }
    
    async fn read_range(&self, start_offset: u64, end_offset: u64) -> anyhow::Result<Vec<LogEntry>> {
        let mut entries = Vec::new();
        
        for offset in start_offset..=end_offset {
            if let Some(entry) = self.read(offset).await? {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    async fn compact(&mut self) -> anyhow::Result<()> {
        // TODO: Implement compaction logic
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_append_and_read() {
        let temp_dir = tempdir().unwrap();
        let mut storage = RocksDbStorage::new(temp_dir.path()).unwrap();
        
        let entry = LogEntry {
            offset: 1,
            timestamp: 1640995200000,
            key: Some("test-key".to_string()),
            value: b"test-value".to_vec(),
        };
        
        let offset = storage.append(entry.clone()).await.unwrap();
        assert_eq!(offset, 1);
        
        let retrieved = storage.read(1).await.unwrap().unwrap();
        assert_eq!(retrieved.key, entry.key);
        assert_eq!(retrieved.value, entry.value);
    }
}
