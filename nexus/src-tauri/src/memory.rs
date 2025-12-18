use std::sync::{Arc, Mutex, OnceLock};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MemoryEntry {
    pub content: String,
    pub tags: Vec<String>,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Memory {
    pub entries: Vec<MemoryEntry>,
}

impl Memory {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, content: String, tags: Vec<String>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.push(MemoryEntry {
            content,
            tags,
            timestamp,
        });
    }

    pub fn get_all(&self) -> Vec<MemoryEntry> {
        self.entries.clone()
    }

    pub fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| {
                entry.content.to_lowercase().contains(&query_lower)
                    || entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
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
        mem.add("test note".to_string(), vec!["tag1".to_string()]);

        let all = mem.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].content, "test note");
        assert_eq!(all[0].tags, vec!["tag1".to_string()]);

        mem.add("another note".to_string(), vec![]);
        assert_eq!(mem.get_all().len(), 2);

        let search_res = mem.search("test");
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].content, "test note");

        let search_res_tag = mem.search("tag1");
        assert_eq!(search_res_tag.len(), 1);

        mem.clear();
        assert!(mem.get_all().is_empty());
    }
}
