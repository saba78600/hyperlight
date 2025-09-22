//! Open-addressing HashMap used for keyword tables and interning.
use std::usize;

const INIT_SIZE: usize = 16;
const HIGH_WATERMARK: usize = 70; // percent
const LOW_WATERMARK: usize = 50; // percent

fn fnv_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= b as u64;
    }
    hash
}

#[derive(Clone)]
struct HashEntry<V: Clone + Default> {
    key: String,
    keylen: usize,
    val: V,
}

#[derive(Clone)]
enum Bucket<V: Clone + Default> {
    Empty,
    Tombstone,
    Occupied(HashEntry<V>),
}

pub struct HashMap<V: Clone + Default> {
    buckets: Vec<Bucket<V>>,
    pub capacity: usize,
    pub used: usize, // number of non-tombstone entries
}

impl<V: Clone + Default> HashMap<V> {
    pub fn new() -> Self {
        HashMap { buckets: Vec::new(), capacity: 0, used: 0 }
    }

    fn rehash(&mut self) {
        // count non-tombstone keys
        let mut nkeys = 0usize;
        for b in &self.buckets {
            if let Bucket::Occupied(_) = b { nkeys += 1; }
        }

        let mut cap = if self.capacity == 0 { INIT_SIZE } else { self.capacity };
        while nkeys * 100 / cap >= LOW_WATERMARK {
            cap = cap.saturating_mul(2);
        }
        if cap == 0 { cap = 1; }

        let mut map2 = HashMap { buckets: vec![Bucket::Empty; cap], capacity: cap, used: 0 };

        for b in &self.buckets {
            if let Bucket::Occupied(ent) = b {
                map2.put2(&ent.key, ent.keylen, ent.val.clone());
            }
        }

        // replace
        *self = map2;
    }

    fn get_entry_index(&self, key: &str, keylen: usize) -> Option<usize> {
        if self.buckets.is_empty() { return None; }
        let hash = fnv_hash(key.as_bytes());
        for i in 0..self.capacity {
            let idx = ((hash as usize) + i) % self.capacity;
            match &self.buckets[idx] {
                Bucket::Occupied(ent) => {
                    if ent.keylen == keylen && ent.key.as_bytes() == &key.as_bytes()[..keylen] {
                        return Some(idx);
                    }
                }
                Bucket::Empty => return None,
                Bucket::Tombstone => continue,
            }
        }
        None
    }

    fn get_or_insert_index(&mut self, key: &str, keylen: usize) -> usize {
        if self.buckets.is_empty() {
            self.buckets = vec![Bucket::Empty; INIT_SIZE];
            self.capacity = INIT_SIZE;
        } else if self.used * 100 / self.capacity >= HIGH_WATERMARK {
            self.rehash();
        }

        let hash = fnv_hash(key.as_bytes());
        for i in 0..self.capacity {
            let idx = ((hash as usize) + i) % self.capacity;
            match &mut self.buckets[idx] {
                Bucket::Occupied(ent) => {
                    if ent.keylen == keylen && ent.key.as_bytes() == &key.as_bytes()[..keylen] {
                        return idx;
                    }
                }
                Bucket::Tombstone => {
                    let s = String::from(&key[..keylen]);
                    self.buckets[idx] = Bucket::Occupied(HashEntry { key: s, keylen, val: V::default() });
                    return idx;
                }
                Bucket::Empty => {
                    let s = String::from(&key[..keylen]);
                    self.buckets[idx] = Bucket::Occupied(HashEntry { key: s, keylen, val: V::default() });
                    self.used += 1;
                    return idx;
                }
            }
        }
        unreachable!();
    }

    pub fn get(&self, key: &str) -> Option<V> {
        self.get2(key, key.len())
    }

    pub fn get2(&self, key: &str, keylen: usize) -> Option<V> {
        self.get_entry_index(key, keylen).map(|idx| {
            if let Bucket::Occupied(ent) = &self.buckets[idx] { ent.val.clone() } else { unreachable!() }
        })
    }

    pub fn put(&mut self, key: &str, val: V) {
        self.put2(key, key.len(), val);
    }

    pub fn put2(&mut self, key: &str, keylen: usize, val: V) {
        let idx = self.get_or_insert_index(key, keylen);
        if let Bucket::Occupied(ent) = &mut self.buckets[idx] {
            ent.val = val;
        }
    }

    pub fn delete(&mut self, key: &str) {
        self.delete2(key, key.len());
    }

    pub fn delete2(&mut self, key: &str, keylen: usize) {
        if let Some(idx) = self.get_entry_index(key, keylen) {
            self.buckets[idx] = Bucket::Tombstone;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HashMap;

    #[test]
    fn smoke() {
        let mut map = HashMap::<usize>::new();
        for i in 0..500 {
            map.put(&format!("key {}", i), i);
        }
        for i in 100..200 {
            map.delete(&format!("key {}", i));
        }
        for i in 150..160 {
            map.put(&format!("key {}", i), i);
        }
        for i in 600..700 {
            map.put(&format!("key {}", i), i);
        }

        for i in 0..100 {
            assert_eq!(map.get(&format!("key {}", i)), Some(i));
        }
        for i in 100..150 {
            assert_eq!(map.get(&format!("key {}", i)), None);
        }
        for i in 150..160 {
            assert_eq!(map.get(&format!("key {}", i)), Some(i));
        }
    }
}
