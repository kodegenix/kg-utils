use std::sync::Arc;
use std::ops::{Deref, DerefMut};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};


pub struct SyncRef<T>(Arc<RwLock<T>>);

impl<T> SyncRef<T> {
    pub fn new(value: T) -> SyncRef<T> {
        SyncRef(Arc::new(RwLock::new(value)))
    }

    #[inline(always)]
    pub fn read(&self) -> SyncRefReadGuard<T> {
        SyncRefReadGuard(self.0.read())
    }

    #[inline(always)]
    pub fn write(&self) -> SyncRefWriteGuard<T> {
        SyncRefWriteGuard(self.0.write())
    }
}

unsafe impl<T> Send for SyncRef<T> {}

unsafe impl<T> Sync for SyncRef<T> {}


pub struct SyncRefReadGuard<'a, T>(RwLockReadGuard<'a, T>);

impl<'a, T> Deref for SyncRefReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}


pub struct SyncRefWriteGuard<'a, T>(RwLockWriteGuard<'a, T>);

impl<'a, T> Deref for SyncRefWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a, T> DerefMut for SyncRefWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test12() {
        let a: SyncRef<String> = SyncRef::new("str".to_string());
        println!("{}", *a.read());

        a.write().push_str("strstr");
        println!("{}", *a.read());
    }
}