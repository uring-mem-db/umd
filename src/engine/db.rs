use std::{collections::HashMap, ptr::NonNull};

pub(crate) trait KeyValueStore<K, V> {
    fn get(&mut self, key: K) -> Option<&V>;
    fn set(&mut self, key: K, value: V);
    fn del(&mut self, key: K);
}

/// Entry is a value that represents a key-value pair in the database. It also is a node of a linked list built
/// while setting values in the database. The linked list is used to implement a LRU cache. In this way we can have
/// a fast access to the most recently used values.
struct Entry {
    key: String,
    value: String,
    prev: Option<NonNull<Entry>>,
    next: Option<NonNull<Entry>>,
}

#[derive(Default)]
pub(crate) struct HashMapDb {
    data: HashMap<String, Entry>,
    head: Option<NonNull<Entry>>,
    tail: Option<NonNull<Entry>>,

    max_items: Option<u64>,
}

impl HashMapDb {
    pub(crate) fn new(max_items: Option<u64>) -> Self {
        let hm = match max_items {
            Some(max_items) => HashMap::with_capacity(max_items as usize),
            None => HashMap::new(),
        };

        Self {
            max_items,
            data: hm,
            ..Default::default()
        }
    }
}

impl KeyValueStore<&str, String> for HashMapDb {
    fn get(&mut self, key: &str) -> Option<&String> {
        let v = self.data.get_mut(key)?;
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
            unsafe {
                (*(v)).prev = self.tail;
                (*(v)).next = None;
            }

            self.tail = Some(v.into());
        }

        Some(&v.value)
    }

    fn set(&mut self, key: &str, value: String) {
        let entry = Entry {
            key: key.to_string(),
            value,
            prev: self.tail,
            next: None,
        };

        if let Some(max) = self.max_items {
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
    }

    fn del(&mut self, key: &str) {
        let mut e = self.data.get_mut(key).unwrap();

        // adjust head to second node if head is the node to be removed
        if self.head == Some(e.into()) {
            self.head = e.next;
        }

        // adjust tail to second last node if tail is the node to be removed
        if self.tail == Some(e.into()) {
            self.tail = e.prev;
        }

        // remove node from the list
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

#[test]
fn lru() {
    let mut db = HashMapDb::new(Some(3));
    db.set("one", "one".to_string());
    db.set("two", "two".to_string());
    db.set("three", "three".to_string());

    db.set("four", "four".to_string());
    let outdated = db.get("one");
    assert_eq!(outdated, None);

    assert_eq!(db.get("two"), Some(&"two".to_string()));
}

    #[test]
    fn linked_list() {
        let mut db = HashMapDb::new(None);

        // first set
        db.set("foo", "bar".to_string());
        assert_eq!(db.get("foo"), Some(&"bar".to_string()));
        assert!(db.tail.is_some());
        assert!(db.head.is_some());
        assert_eq!(db.tail, db.head);

        // second set
        db.set("foz", "baz".to_string());
        assert_eq!(db.get("foz"), Some(&"baz".to_string()));
        assert!(db.tail.is_some());
        assert!(db.head.is_some());
        assert_ne!(db.tail, db.head);
        unsafe {
            assert_eq!((*(db.tail.unwrap().as_ptr())).value, "baz".to_string());
            assert_eq!((*(db.head.unwrap().as_ptr())).value, "bar".to_string());
        }

        // get first key, it should be the most recently used now then moved to the tail
        let output = db.get("foo");
        assert_eq!(output, Some(&"bar".to_string()));
        assert_ne!(db.tail, db.head);
        unsafe {
            assert_eq!((*(db.tail.unwrap().as_ptr())).value, "bar".to_string());
            assert_eq!((*(db.head.unwrap().as_ptr())).value, "baz".to_string());
        }

        // set a third key
        db.set("fob", "bax".to_string());
        assert_eq!(db.get("fob"), Some(&"bax".to_string()));
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

        // antoher remove
        db.del("foz");
        assert_eq!(db.tail, db.head);

        // remove last one
        db.del("fob");
        assert_eq!(db.tail, None);
        assert_eq!(db.head, None);
    }
}
