use mccfr::*;

/// A concurrent read-through cache. Misses compute outside the lock, so parallel traversals
/// never serialize on a forward pass; two threads may compute the same miss, and one result wins.
pub struct Memo<K, V>
where
    K: Copy + Eq + std::hash::Hash,
{
    shards: Vec<std::sync::RwLock<FxHashMap<K, V>>>,
}

/// One prediction per infoset, held until the net changes.
pub type Predictions<N> = Memo<InfoOf<N>, Over<EdgeOf<N>, Policy>>;

impl<K, V> Default for Memo<K, V>
where
    K: Copy + Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self {
            shards: (0..Self::SHARDS).map(|_| std::sync::RwLock::default()).collect(),
        }
    }
}

impl<K, V> Memo<K, V>
where
    K: Copy + Eq + std::hash::Hash,
{
    /// Independent locks, so rayon workers reading different infosets rarely share a cache line.
    const SHARDS: usize = 1 << 4;

    /// `f` of the cached value at `key`, computing it with `miss` on first sight.
    pub fn with<R>(&self, key: &K, miss: impl FnOnce() -> V, f: impl FnOnce(&V) -> R) -> R {
        let shard = self.shard(key);
        if let Some(value) = shard.read().expect("memo lock").get(key) {
            return f(value);
        }
        let value = miss();
        f(shard.write().expect("memo lock").entry(*key).or_insert(value))
    }

    /// Exclusive access needs no lock. Skips empty tables, whose `clear` still sweeps capacity.
    pub fn clear(&mut self) {
        self.shards_mut()
            .iter_mut()
            .map(|shard| shard.get_mut().expect("memo lock"))
            .filter(|table| !table.is_empty())
            .for_each(FxHashMap::clear);
    }

    pub fn len(&self) -> usize {
        self.shards()
            .iter()
            .map(|shard| shard.read().expect("memo lock").len())
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Middle bits: each shard's table indexes buckets by the low bits of the same hash.
    fn shard(&self, key: &K) -> &std::sync::RwLock<FxHashMap<K, V>> {
        use std::hash::BuildHasher;
        &self.shards()[(rustc_hash::FxBuildHasher.hash_one(key) >> 32) as usize % Self::SHARDS]
    }

    fn shards(&self) -> &[std::sync::RwLock<FxHashMap<K, V>>] {
        &self.shards
    }

    fn shards_mut(&mut self) -> &mut [std::sync::RwLock<FxHashMap<K, V>>] {
        &mut self.shards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn misses_compute_once() {
        let memo = Memo::<u8, u8>::default();
        let calls = std::cell::Cell::new(0);
        let miss = || {
            calls.set(calls.get() + 1);
            7
        };
        assert_eq!(memo.with(&1, miss, |&v| v), 7);
        assert_eq!(memo.with(&1, miss, |&v| v), 7);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn clear_forgets() {
        let mut memo = Memo::<u8, u8>::default();
        memo.with(&1, || 7, |_| ());
        memo.clear();
        assert!(memo.is_empty());
        assert_eq!(memo.with(&1, || 9, |&v| v), 9);
    }
}
