use std::vec::IntoIter;
use std::ops::Deref;


/// Simple implementation of an ordered set, using `std::vec::Vec<_>` as underlying storage.
///
/// Elements in the set are ordered and unique.
/// This collection is efficient only for relatively small number of elements as the cost of
/// insert into underlying `Vec<_>` is O(n).
/// Element lookup is performed with binary search algorithm.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrdSet<T: Ord>(Vec<T>);

impl<T: Ord> OrdSet<T> {
    pub fn new() -> OrdSet<T> {
        OrdSet(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> OrdSet<T> {
        OrdSet(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn contains(&self, elem: &T) -> bool {
        self.0.binary_search(elem).is_ok()
    }

    #[inline]
    pub fn insert(&mut self, elem: T) -> bool {
        match self.0.binary_search(&elem) {
            Ok(index) => {
                self.0[index] = elem;
                true
            }
            Err(index) => {
                self.0.insert(index, elem);
                false
            }
        }
    }

    #[inline]
    pub fn remove(&mut self, elem: &T) -> bool {
        if let Ok(index) = self.0.binary_search(elem) {
            self.0.remove(index);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl<T: Ord> IntoIterator for OrdSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Deref for OrdSet<T>
    where T: Ord
{
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.0.deref()
    }
}

impl<T> std::fmt::Debug for OrdSet<T>
    where T: Ord + std::fmt::Debug
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_set().entries(self.0.iter()).finish()
    }
}

#[cfg(feature = "serde_impl")]
mod serde {
    extern crate serde;

    use super::*;
    use self::serde::{ser, de};

    impl<T> ser::Serialize for OrdSet<T>
        where T: Ord + ser::Serialize
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: ser::Serializer
        {
            use self::ser::SerializeSeq;

            let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
            for e in self.0.iter() {
                seq.serialize_element(e)?;
            }
            seq.end()
        }
    }

    impl<'de, T> de::Deserialize<'de> for OrdSet<T>
        where T: Ord + de::Deserialize<'de>
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: de::Deserializer<'de>
        {
            let mut elems: Vec<T> = Vec::deserialize(deserializer)?;
            elems.sort();
            Ok(OrdSet(elems))
        }
    }
}