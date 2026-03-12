// chimera-state/src/lib.rs
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

pub struct StateManager {
    storage_path: PathBuf,
}

impl StateManager {
    pub fn save<T: Serialize>(&self, key: &str, data: &T) -> Result<(), StateError> {
        // Implement sled/rocksdb storage here
        Ok(())
    }

    pub fn load<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, StateError> {
        // Implement retrieval here
        Err(StateError::NotFound)
    }
}