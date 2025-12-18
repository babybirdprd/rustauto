use std::sync::{Arc, Mutex, OnceLock};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Memory {
    pub notes: Vec<String>,
}

impl Memory {
    pub fn new() -> Self {
        Self { notes: Vec::new() }
    }

    pub fn add(&mut self, note: String) {
        self.notes.push(note);
    }

    pub fn get_all(&self) -> Vec<String> {
        self.notes.clone()
    }

    pub fn clear(&mut self) {
        self.notes.clear();
    }
}

pub static GLOBAL_MEMORY: OnceLock<Arc<Mutex<Memory>>> = OnceLock::new();

pub fn init_memory() {
    let _ = GLOBAL_MEMORY.set(Arc::new(Mutex::new(Memory::new())));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory() {
        let mut mem = Memory::new();
        mem.add("test".to_string());
        assert_eq!(mem.get_all(), vec!["test".to_string()]);

        mem.add("test2".to_string());
        assert_eq!(mem.get_all(), vec!["test".to_string(), "test2".to_string()]);

        mem.clear();
        assert!(mem.get_all().is_empty());
    }
}
