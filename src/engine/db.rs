use crate::config;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;

#[allow(clippy::module_name_repetitions)]
pub fn create_db(c: &config::Engine) -> Result<Rc<RefCell<HashMapDb>>, std::io::Error> {
    let db = Rc::new(RefCell::new(HashMapDb::new(c.clone())));

    if let Some(p) = &c.persistence {
        if !p.enabled {
            return Ok(db);
        }

        let data = std::fs::read_to_string(&p.file);
        if data.is_err() {
            return Ok(db);
        }

        let d = bincode::deserialize::<HashMapDb>(data?.as_bytes()).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("error on deserializing db: {e}"),
            )
        })?;

        // disable persistence to avoid infinite loop
        db.borrow_mut().config.persistence = None;

        // fill db with data
        d.data.iter().for_each(|(k, v)| {
            db.borrow_mut().set(k, v.value.clone(), None);
        });

        // inject config
        db.borrow_mut().config = c.clone();

        tracing::info!("loaded db from {}", p.file);
    }

    Ok(db)
}

/// Entry is a value that represents a key-value pair in the database. It also is a node of a linked list built
/// while setting values in the database. The linked list is used to implement LRU cache.
/// In this way we can have fast access to the most recently used values.
#[derive(serde::Deserialize, serde::Serialize)]
struct Entry {
    key: String,
    value: String,

    #[serde(skip_serializing, skip_deserializing)]
    prev: Option<NonNull<Entry>>,

    #[serde(skip_serializing, skip_deserializing)]
    next: Option<NonNull<Entry>>,
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct HashMapDb {
    data: HashMap<String, Entry>,

    #[serde(skip_serializing, skip_deserializing)]
    head: Option<NonNull<Entry>>,

    #[serde(skip_serializing, skip_deserializing)]
    tail: Option<NonNull<Entry>>,

    #[serde(skip_serializing, skip_deserializing)]
    ttl: HashMap<String, std::time::Instant>,

    #[serde(skip_serializing, skip_deserializing)]
    config: config::Engine,

    #[serde(skip_serializing, skip_deserializing)]
    changes: u64,
}


impl HashMapDb {
    pub(crate) fn new(conf: config::Engine) -> Self {
        let hm = conf.max_items.map_or_else(HashMap::new, |max_items| {
            HashMap::with_capacity(usize::try_from(max_items).unwrap())
        });

        Self {
            data: hm,
            config: conf,
            ..Default::default()
        }
    }

    /// Check the command [`FlushDb`](protocol::commands::Command::FlushDb) for more details.
    pub fn flush(&mut self) {
        let c = self.config.clone();
        *self = Self::new(c);
    }

    /// Check if we have persistence enabled and if we have it + we reach the threshold of changes,
    /// we should persist the data to disk.
    fn evaluate_update_persistence(&mut self) {
        if let Some(persistence) = &self.config.persistence {
            self.changes += 1;
            if self.changes >= persistence.flush_every_changes {
                self.changes = 0;
                self.persist();
            }
        }
    }

    fn persist(&self) {
        if let Some(persistence) = &self.config.persistence {
            if !persistence.enabled {
                return;
            }

            let s = bincode::serialize(&self).unwrap();
            tracing::info!("persisting db to {}", persistence.file);
            std::fs::File::create(&persistence.file).unwrap();
            std::fs::write(&persistence.file, s).unwrap();
        }
    }

    pub fn exists(&mut self, key: &str, instant: std::time::Instant) -> bool {
        self.get(key, instant).is_some()
    }

    pub fn get(&mut self, key: &str, now: std::time::Instant) -> Option<&String> {
        let v = match self.ttl.get(key) {
            Some(ttl) if *ttl <= now => {
                self.del(key);

                None
            }
            _ => self.data.get_mut(key),
        }?;

        if self.tail != self.head && self.tail != Some(v.into()) {
            // adjust head to second node if head is the node to be removed
            if self.head == Some(v.into()) {
                self.head = v.next;
            }

            // remove node from the list
            if let Some(prev) = v.prev {
                unsafe {
                    (*prev.as_ptr()).next = v.next;
                }
            }
            if let Some(next) = v.next {
                unsafe {
                    (*next.as_ptr()).prev = v.prev;
                }
            }

            // attach node to the tail
            if let Some(tail) = self.tail {
                unsafe {
                    (*tail.as_ptr()).next = Some(v.into());
                }
            }

            v.prev = self.tail;
            v.next = None;

            self.tail = Some(v.into());
        }

        Some(&v.value)
    }

    pub fn set(&mut self, key: &str, value: String, ttl: Option<std::time::Instant>) {
        let entry = Entry {
            key: key.to_string(),
            value,
            prev: self.tail,
            next: None,
        };

        if let Some(max) = self.config.max_items {
            if self.data.len() as u64 == max {
                let h = self.head.as_ref().unwrap();
                unsafe {
                    self.data.remove(&(*h.as_ptr()).key);
                }
            }
        }

        self.data.insert(key.to_string(), entry);
        let e = self.data.get(key).unwrap();

        if self.head.is_none() {
            self.head = Some(e.into());
        }

        if let Some(t) = self.tail {
            unsafe {
                (*t.as_ptr()).next = Some(e.into());
            }
        }

        self.tail = Some(e.into());

        if let Some(ttl) = ttl {
            self.ttl.insert(key.to_string(), ttl);
        }

        self.evaluate_update_persistence();
    }

    pub fn del(&mut self, key: &str) {
        let e = self.data.get_mut(key).unwrap();

        // adjust head to second node if head is the node to be removed
        if self.head == Some(e.into()) {
            self.head = e.next;
        }

        // adjust tail to latest - 1 node if tail is the node to be removed
        if self.tail == Some(e.into()) {
            self.tail = e.prev;
        }

        // remove node from the list in case is not the head or tail
        if let Some(prev) = e.prev {
            unsafe {
                (*prev.as_ptr()).next = e.next;
            }
        }
        if let Some(next) = e.next {
            unsafe {
                (*next.as_ptr()).prev = e.prev;
            }
        }

        self.data.remove(key);
        self.ttl.remove(key);

        self.evaluate_update_persistence();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flush() {
        let mut db = HashMapDb::new(config::Engine {
            max_items: Some(3),
            ..Default::default()
        });
        db.set("one", "one".to_string(), None);
        db.set("two", "two".to_string(), None);
        db.set("three", "three".to_string(), None);

        db.flush();

        assert_eq!(db.get("one", std::time::Instant::now()), None);
        assert_eq!(db.get("two", std::time::Instant::now()), None);
        assert_eq!(db.get("three", std::time::Instant::now()), None);
    }

    #[test]
    fn lru() {
        let mut db = HashMapDb::new(config::Engine {
            max_items: Some(3),
            ..Default::default()
        });
        db.set("one", "one".to_string(), None);
        db.set("two", "two".to_string(), None);
        db.set("three", "three".to_string(), None);

        db.set("four", "four".to_string(), None);
        let outdated = db.get("one", std::time::Instant::now());
        assert_eq!(outdated, None);

        assert_eq!(
            db.get("two", std::time::Instant::now()),
            Some(&"two".to_string())
        );
    }

    #[test]
    fn linked_list() {
        let mut db = HashMapDb::new(config::Engine::default());

        // first set
        db.set("foo", "bar".to_string(), None);
        assert_eq!(
            db.get("foo", std::time::Instant::now()),
            Some(&"bar".to_string())
        );
        assert!(db.tail.is_some());
        assert!(db.head.is_some());
        assert_eq!(db.tail, db.head);

        // second set
        db.set("foz", "baz".to_string(), None);
        assert_eq!(
            db.get("foz", std::time::Instant::now()),
            Some(&"baz".to_string())
        );
        assert!(db.tail.is_some());
        assert!(db.head.is_some());
        assert_ne!(db.tail, db.head);
        unsafe {
            assert_eq!((*(db.tail.unwrap().as_ptr())).value, "baz".to_string());
            assert_eq!((*(db.head.unwrap().as_ptr())).value, "bar".to_string());
        }

        // get first key, it should be the most recently used now then moved to the tail
        let output = db.get("foo", std::time::Instant::now());
        assert_eq!(output, Some(&"bar".to_string()));
        assert_ne!(db.tail, db.head);
        unsafe {
            assert_eq!((*(db.tail.unwrap().as_ptr())).value, "bar".to_string());
            assert_eq!((*(db.head.unwrap().as_ptr())).value, "baz".to_string());
        }

        // set a third key
        db.set("fob", "bax".to_string(), None);
        assert_eq!(
            db.get("fob", std::time::Instant::now()),
            Some(&"bax".to_string())
        );
        assert_ne!(db.tail, db.head);
        unsafe {
            assert_eq!((*(db.tail.unwrap().as_ptr())).value, "bax".to_string());
            assert_eq!((*(db.head.unwrap().as_ptr())).value, "baz".to_string());
        }

        // remove the first key
        db.del("foo");
        assert_ne!(db.tail, db.head);
        unsafe {
            assert_eq!((*(db.tail.unwrap().as_ptr())).value, "bax".to_string());
            assert_eq!((*(db.head.unwrap().as_ptr())).value, "baz".to_string());
        }

        // another remove
        db.del("foz");
        assert_eq!(db.tail, db.head);

        // remove last one
        db.del("fob");
        assert_eq!(db.tail, None);
        assert_eq!(db.head, None);
    }

    #[test]
    fn lazy_ttl() {
        let mut db = HashMapDb::new(config::Engine::default());
        let now = std::time::Instant::now();
        db.set(
            "foo",
            "bar".to_string(),
            Some(now + std::time::Duration::from_secs(10)),
        );
        assert_eq!(
            db.get("foo", now + std::time::Duration::from_secs(1)),
            Some(&"bar".to_string())
        );
        assert_eq!(
            db.get("foo", now + std::time::Duration::from_secs(11)),
            None
        );
    }

    #[test]
    fn serialize_entry() {
        let e = Entry {
            key: "foo".to_string(),
            value: "bar".to_string(),
            prev: None,
            next: None,
        };

        let s = bincode::serialize(&e).unwrap();
        assert_eq!(s.len(), 22);

        let ee = bincode::deserialize::<Entry>(&s).unwrap();
        assert_eq!(e.key, ee.key);
        assert_eq!(e.value, ee.value);
    }

    #[test]
    fn serialize_db() {
        let mut db = HashMapDb::new(config::Engine::default());
        db.set("foo", "bar".to_string(), None);
        db.set("baz", "qux".to_string(), None);

        let s = bincode::serialize(&db).unwrap();
        assert_eq!(s.len(), 74);

        let mut dd = bincode::deserialize::<HashMapDb>(&s).unwrap();
        assert_eq!(
            dd.get("foo", std::time::Instant::now()),
            Some(&"bar".to_string())
        );
        assert_eq!(
            dd.get("baz", std::time::Instant::now()),
            Some(&"qux".to_string())
        );
    }

    #[test]
    fn persist() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let file_path = file.path().to_str().unwrap().to_string();
        let c = config::Engine {
            persistence: Some(config::Persistence {
                enabled: true,
                flush_every_changes: 2,
                file: file_path.clone(),
            }),
            ..Default::default()
        };
        let mut db = HashMapDb::new(c.clone());

        {
            // first 2 changes
            db.set("one", "one".to_string(), None);
            db.set("two", "two".to_string(), None);

            let dd = create_db(&c).unwrap();
            assert_eq!(
                dd.borrow_mut().get("one", std::time::Instant::now()),
                Some(&"one".to_string())
            );
            assert_eq!(
                dd.borrow_mut().get("two", std::time::Instant::now()),
                Some(&"two".to_string())
            );
        }
        {
            // another 2 changes
            db.del("one");
            db.set("three", "three".to_string(), None);

            let dd = create_db(&c).unwrap();

            assert_eq!(dd.borrow_mut().get("one", std::time::Instant::now()), None);
            assert_eq!(
                dd.borrow_mut().get("two", std::time::Instant::now()),
                Some(&"two".to_string())
            );
            assert_eq!(
                dd.borrow_mut().get("three", std::time::Instant::now()),
                Some(&"three".to_string())
            );
        }
    }
}
