use std::collections::HashMap;

pub(crate) trait KeyValueStore<K, V> {
    fn get(&self, key: K) -> Option<&V>;
    fn set(&mut self, key: K, value: V);
    fn del(&mut self, key: K);
}

#[derive(Default)]
pub(crate) struct HashMapDb {
    data: HashMap<String, String>,
}

impl HashMapDb {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl KeyValueStore<&str, String> for HashMapDb {
    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }

    fn del(&mut self, key: &str) {
        self.data.remove(key);
    }
}
