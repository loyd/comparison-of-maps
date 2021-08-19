use std::collections;
use std::hash::Hash;

use fxhash::FxBuildHasher;
use packed_simd::{u32x16, u32x8};

pub trait Map<K, V> {
    fn insert(&mut self, key: K, value: V);
    fn find(&self, key: K) -> Option<&V>;
}

#[derive(Default)]
pub struct LinearMap<K, V> {
    data: Vec<(K, V)>,
}

impl<K: Eq, V> Map<K, V> for LinearMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.data.push((key, value));
    }

    fn find(&self, key: K) -> Option<&V> {
        self.data
            .iter()
            .find(|entry| entry.0 == key)
            .map(|entry| &entry.1)
    }
}

#[derive(Default)]
pub struct BinaryMap<K, V> {
    data: Vec<(K, V)>,
}

impl<K: Ord, V> Map<K, V> for BinaryMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.data.push((key, value));
        self.data.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    }

    fn find(&self, key: K) -> Option<&V> {
        match self.data.binary_search_by(|entry| entry.0.cmp(&key)) {
            Ok(index) => Some(unsafe { &self.data.get_unchecked(index).1 }),
            Err(_) => None,
        }
    }
}

#[derive(Default)]
pub struct KvMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Eq, V> Map<K, V> for KvMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }

    fn find(&self, key: K) -> Option<&V> {
        let index = self.keys.iter().position(|stored| stored == &key)?;
        Some(unsafe { self.values.get_unchecked(index) })
    }
}

#[derive(Default)]
pub struct SimdMap16<K, V> {
    keys: Vec<K>,
    next: usize,
    values: Vec<V>,
}

impl<V> Map<u32, V> for SimdMap16<u32, V> {
    fn insert(&mut self, key: u32, value: V) {
        if self.next == self.keys.len() {
            self.keys.extend(&[0; 16]);
        }

        self.keys[self.next] = key;
        self.next += 1;
        self.values.push(value);
    }

    fn find(&self, key: u32) -> Option<&V> {
        for index in (0..self.keys.len()).step_by(16) {
            let cursor = &self.keys[index..];
            let mask = u32x16::from_slice_unaligned(cursor).eq(u32x16::splat(key));
            let zeros = mask.bitmask().trailing_zeros();

            if zeros < 16 {
                return self.values.get(index + zeros as usize);
            }
        }

        None
    }
}

#[derive(Default)]
pub struct SimdMap8<K, V> {
    keys: Vec<K>,
    next: usize,
    values: Vec<V>,
}

impl<V> Map<u32, V> for SimdMap8<u32, V> {
    fn insert(&mut self, key: u32, value: V) {
        if self.next == self.keys.len() {
            self.keys.extend(&[0; 8]);
        }

        self.keys[self.next] = key;
        self.next += 1;
        self.values.push(value);
    }

    fn find(&self, key: u32) -> Option<&V> {
        for index in (0..self.keys.len()).step_by(8) {
            let cursor = &self.keys[index..];
            let mask = u32x8::from_slice_unaligned(cursor).eq(u32x8::splat(key));
            let zeros = mask.bitmask().trailing_zeros();

            if zeros < 8 {
                return self.values.get(index + zeros as usize);
            }
        }

        None
    }
}

pub type HashMap<K, V> = collections::HashMap<K, V, FxBuildHasher>;

#[allow(clippy::implicit_hasher)]
impl<K: Eq + Hash, V> Map<K, V> for HashMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    fn find(&self, key: K) -> Option<&V> {
        self.get(&key)
    }
}

pub type BTreeMap<K, V> = collections::BTreeMap<K, V>;

impl<K: Ord, V> Map<K, V> for BTreeMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    fn find(&self, key: K) -> Option<&V> {
        self.get(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run<M: Map<u32, i32> + Default>() {
        let mut map = M::default();
        assert_eq!(map.find(8), None);

        map.insert(5, 42);
        map.insert(1, 43);
        map.insert(7, 44);

        assert_eq!(map.find(5), Some(&42));
        assert_eq!(map.find(1), Some(&43));
        assert_eq!(map.find(7), Some(&44));
        assert_eq!(map.find(8), None);
    }

    #[test]
    fn linear_map() {
        run::<LinearMap<_, _>>();
    }

    #[test]
    fn binary_map() {
        run::<BinaryMap<_, _>>();
    }

    #[test]
    fn kv_map() {
        run::<KvMap<_, _>>();
    }

    #[test]
    fn simd_map() {
        run::<SimdMap16<_, _>>();
        run::<SimdMap8<_, _>>();
    }

    #[test]
    fn hash_map() {
        run::<HashMap<_, _>>();
    }

    #[test]
    fn btree_map() {
        run::<BTreeMap<_, _>>();
    }
}
