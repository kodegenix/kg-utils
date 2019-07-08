extern crate heapsize;

use std::hash::{BuildHasher, Hash};

use super::LruCache;

use self::heapsize::HeapSizeOf;

impl<K: Eq + Hash + HeapSizeOf, V: HeapSizeOf, S: BuildHasher> HeapSizeOf for LruCache<K, V, S> {
    fn heap_size_of_children(&self) -> usize {
        self.map.heap_size_of_children()
    }
}
