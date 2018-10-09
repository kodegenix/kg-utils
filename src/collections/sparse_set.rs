use super::*;

use std::alloc::{Layout, alloc, realloc, dealloc, handle_alloc_error};
use std::convert::{TryFrom, TryInto};
use std::collections::VecDeque;
use std::ops::Deref;

use serde::{ser, de};


/// A very fast insertion/lookup set type `SparseSet<T>`, with stable insertion-order iteration,
/// for array index-like element types.
///
/// This implementation is very fast and cheap for insertions/lookups, however there are rather
/// severe restrictions on the element type `T`:
/// * `T` **must not** implement `Drop`. Implementation sometimes reads elements from uninitialized
/// memory, and therefore elements cannot be reliably dropped.
/// * `T` must implement `Copy`.
/// * `T` must be convertible to and from `usize`, where the converted `usize` value must always
/// lie in the range from 0 to set capacity (see [capacity()][Self::capacity()] method).
///
/// Pretty much the only sensible choice for `T` is one of primitive integral types, possibly
/// wrapped in a simple wrapper type, if needed.
/// In general this set type is most useful for lookup algorithms, to store indices of an input
/// collection that fulfill a specific conditions.
pub struct SparseSet<T: TryFrom<usize> + TryInto<usize> + Copy> {
    capacity: usize,
    len: usize,
    dense: *mut T,
    sparse: *mut T,
}

impl<T: TryFrom<usize> + TryInto<usize> + Copy> SparseSet<T> {
    pub fn new(size: usize) -> SparseSet<T> {
        SparseSet {
            capacity: size,
            len: 0,
            dense: unsafe { mem_alloc(size) },
            sparse: unsafe { mem_alloc(size) },
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn resize(&mut self, size: usize) {
        if self.capacity < size {
            self.dense = unsafe { mem_realloc(self.dense, self.capacity, size) };
            self.sparse = unsafe { mem_realloc(self.sparse, self.capacity, size) };
        }
        self.capacity = size;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn insert(&mut self, value: T) {
        let val = to_usize(value);
        if val >= self.capacity {
            panic!("value out of range");
        } else if !self.contains(&value) {
            let i = self.len;
            unsafe {
                std::ptr::write(self.dense.offset(i as isize), value);
                std::ptr::write(self.sparse.offset(val as isize), to_value(i));
            }
            self.len += 1;
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        let val = to_usize(*value);
        let i = to_usize(unsafe { std::ptr::read(self.sparse.offset(val as isize)) });
        if i < self.len {
            let j = to_usize(unsafe { std::ptr::read(self.dense.offset(i as isize)) });
            val == j
        } else {
            false
        }
    }
}

impl<T: TryFrom<usize> + TryInto<usize> + Copy> Clone for SparseSet<T> {
    fn clone(&self) -> Self {
        let mut s = SparseSet::new(self.capacity);
        unsafe {
            std::ptr::copy_nonoverlapping(self.dense, s.dense, self.len);
            std::ptr::copy_nonoverlapping(self.sparse, s.sparse, self.capacity);
        }
        s.len = self.len;
        s
    }
}

impl<T: TryFrom<usize> + TryInto<usize> + Copy> Drop for SparseSet<T> {
    fn drop(&mut self) {
        unsafe {
            mem_dealloc(self.dense, self.capacity);
            mem_dealloc(self.sparse, self.capacity);
        }
    }
}

impl<T: TryFrom<usize> + TryInto<usize> + Copy> Deref for SparseSet<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.dense, self.len) }
    }
}

impl<T: TryFrom<usize> + TryInto<usize> + Copy + std::fmt::Debug> std::fmt::Debug for SparseSet<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_set().entries(self.deref().iter()).finish()
    }
}

impl<A, B> PartialEq<SparseSet<B>> for SparseSet<A>
where A: TryFrom<usize> + TryInto<usize> + Copy + PartialEq<B>,
      B: TryFrom<usize> + TryInto<usize> + Copy
{
    fn eq(&self, other: &SparseSet<B>) -> bool {
        if self.len() == other.len() {
            for (a, b) in self.iter().zip(other.iter()) {
                if a != b {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl<A, B> PartialEq<Vec<B>> for SparseSet<A>
    where A: TryFrom<usize> + TryInto<usize> + Copy + PartialEq<B>,
          B: TryFrom<usize> + TryInto<usize> + Copy
{
    fn eq(&self, other: &Vec<B>) -> bool {
        if self.len() == other.len() {
            for (a, b) in self.iter().zip(other.iter()) {
                if a != b {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl<A, B> PartialEq<VecDeque<B>> for SparseSet<A>
where A: TryFrom<usize> + TryInto<usize> + Copy + PartialEq<B>,
      B: TryFrom<usize> + TryInto<usize> + Copy
{
    fn eq(&self, other: &VecDeque<B>) -> bool {
        if self.len() == other.len() {
            for (a, b) in self.iter().zip(other.iter()) {
                if a != b {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

unsafe impl<T: TryFrom<usize> + TryInto<usize> + Copy> Send for SparseSet<T> {}

impl<T> ser::Serialize for SparseSet<T>
    where T: TryFrom<usize> + TryInto<usize> + Copy + ser::Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        use self::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'de, T> de::Deserialize<'de> for SparseSet<T>
    where T: TryFrom<usize> + TryInto<usize> + Copy + de::Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        let elems: Vec<T> = Vec::deserialize(deserializer)?;

        let mut max = 0;
        for e in elems.iter().cloned() {
            max = std::cmp::max(max, to_usize(e));
        }

        let mut set = SparseSet::new(max);
        for e in elems {
            set.insert(e);
        }

        Ok(set)
    }
}


#[inline]
fn layout<T>(size: usize) -> Layout {
    match Layout::array::<T>(size) {
        Ok(layout) => layout,
        Err(err) => panic!(err.to_string()),
    }
}

#[inline]
unsafe fn mem_alloc<T>(size: usize) -> *mut T {
    let l = layout::<T>(size);

    let ptr = alloc(l);
    if ptr.is_null() {
        handle_alloc_error(l);
    } else {
        ptr as *mut T
    }
}

#[inline]
unsafe fn mem_realloc<T>(ptr: *mut T, size: usize, new_size: usize) -> *mut T {
    let l = layout::<T>(size);

    let ptr = realloc(ptr as *mut u8, l, layout::<T>(new_size).size());
    if ptr.is_null() {
        handle_alloc_error(l);
    } else {
        ptr as *mut T
    }
}

#[inline]
unsafe fn mem_dealloc<T>(ptr: *mut T, size: usize) {
    let l = layout::<T>(size);

    dealloc(ptr as *mut u8, l);
}


#[inline]
fn to_usize<T: TryFrom<usize> + TryInto<usize> + Copy>(value: T) -> usize {
    match value.try_into() {
        Ok(v) => v,
        Err(_) => panic!("conversion failed"),
    }
}

#[inline]
fn to_value<T: TryFrom<usize> + TryInto<usize> + Copy>(value: usize) -> T {
    match T::try_from(value) {
        Ok(v) => v,
        Err(_) => panic!("conversion failed"),
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use test::Bencher;
    use super::*;

    #[test]
    fn values_are_unique() {
        let mut set = SparseSet::new(1024);
        let mut count = 0;
        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                set.insert(i);
                count += 1;
            }
        }

        // This should not add any more elements
        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                set.insert(i);
            }
        }

        assert_eq!(set.len(), count);

        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                assert_eq!(set.contains(&i), true);
            } else {
                assert_eq!(set.contains(&i), false);
            }
        }
    }

    #[test]
    fn iterate_in_insertion_order() {
        let mut set = SparseSet::new(1024);
        let mut vec = Vec::with_capacity(1024);
        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                set.insert(i);
                vec.push(i);
            }
        }

        for (s, v) in set.iter().zip(vec.iter()) {
            assert_eq!(s, v);
        }
    }

    #[test]
    fn clone_makes_deep_copy() {
        let set = {
            let mut set = SparseSet::new(1024);
            for i in 0u16..1024u16 {
                if (i % 3) == 0 {
                    set.insert(i);
                }
            }
            set.clone()
        };

        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                assert_eq!(set.contains(&i), true);
            } else {
                assert_eq!(set.contains(&i), false);
            }
        }
    }

    #[bench]
    fn sparse_set_insert_bench(b: &mut Bencher) {
        let mut set = SparseSet::new(1024);

        b.iter(|| {
            set.clear();
            for i in 0u16..1024u16 {
                if (i % 3) == 0 {
                    set.insert(i);
                }
            }
        });
    }

    #[bench]
    fn hash_set_insert_bench(b: &mut Bencher) {
        let mut set = HashSet::with_capacity(1024);

        b.iter(|| {
            set.clear();
            for i in 0u16..1024u16 {
                if (i % 3) == 0 {
                    set.insert(i);
                }
            }
        });
    }

    #[bench]
    fn vec_insert_bench(b: &mut Bencher) {
        let mut set = Vec::with_capacity(1024);

        b.iter(|| {
            set.clear();
            for i in 0u16..1024u16 {
                if (i % 3) == 0 {
                    set.push(i);
                }
            }
        });
    }

    #[bench]
    fn sparse_set_contains_bench(b: &mut Bencher) {
        let mut set = SparseSet::new(1024);
        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                set.insert(i);
            }
        }

        let mut count = 0;
        b.iter(|| {
            for i in 0u16..1024u16 {
                if set.contains(&i) {
                    count += 1;
                }
            }
        });
    }

    #[bench]
    fn hash_set_contains_bench(b: &mut Bencher) {
        let mut set = HashSet::with_capacity(1024);
        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                set.insert(i);
            }
        }

        let mut count = 0;
        b.iter(|| {
            for i in 0u16..1024u16 {
                if set.contains(&i) {
                    count += 1;
                }
            }
        });
    }

    #[bench]
    fn vec_contains_bench(b: &mut Bencher) {
        let mut set = Vec::with_capacity(1024);
        for i in 0u16..1024u16 {
            if (i % 3) == 0 {
                set.push(i);
            }
        }

        let mut count = 0;
        b.iter(|| {
            for i in 0u16..1024u16 {
                if set.contains(&i) {
                    count += 1;
                }
            }
        });
    }
}