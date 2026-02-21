use std::{collections::HashMap, fs, io, path::Path};

#[derive(Debug, Default)]
pub struct MemoryStore {
    entries: HashMap<String, String>,
}

impl MemoryStore {
    pub fn put(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    pub fn save_encrypted(&self, path: impl AsRef<Path>, key: u8) -> io::Result<()> {
        let mut lines = Vec::new();
        for (k, v) in &self.entries {
            lines.push(format!("{k}={v}"));
        }
        let plaintext = lines.join("\n");
        let encrypted: Vec<u8> = plaintext.bytes().map(|b| b ^ key).collect();
        fs::write(path, encrypted)
    }

    pub fn load_encrypted(path: impl AsRef<Path>, key: u8) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        let plaintext: String = bytes.into_iter().map(|b| (b ^ key) as char).collect();
        let mut entries = HashMap::new();
        for line in plaintext.lines() {
            if let Some((k, v)) = line.split_once('=') {
                entries.insert(k.to_string(), v.to_string());
            }
        }
        Ok(Self { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_roundtrip_encrypted_file() {
        let mut store = MemoryStore::default();
        store.put("a", "1");
        let path = "memory_test.enc";

        store
            .save_encrypted(path, 7)
            .expect("encrypted memory should save");
        let loaded = MemoryStore::load_encrypted(path, 7).expect("encrypted memory should load");
        std::fs::remove_file(path).expect("cleanup should succeed");

        assert_eq!(loaded.get("a"), Some("1"));
    }
}
