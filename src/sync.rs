use std::sync::Arc;
use std::ops::{Deref, DerefMut};
use std::thread::ThreadId;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard, Mutex};

const DEADLOCK_MSG: &str = "deadlock detected, lock already acquired in the current thread";


pub use sync_ref::{SyncRef, SyncRefReadGuard, SyncRefWriteGuard};

unsafe impl<T> Send for SyncRef<T> {}

unsafe impl<T> Sync for SyncRef<T> {}


#[cfg(debug_assertions)]
mod sync_ref {
    use super::*;
    use std::mem::ManuallyDrop;

    pub struct SyncRef<T>(Arc<RwLockDbg<T>>);

    impl<T> SyncRef<T> {
        pub fn new(value: T) -> SyncRef<T> {
            SyncRef(Arc::new(RwLockDbg::new(value)))
        }

        #[inline(always)]
        pub fn read(&self) -> SyncRefReadGuard<T> {
            self.0.prepare_lock();
            let guard = self.0.lock.read();
            SyncRefReadGuard {
                guard: ManuallyDrop::new(guard),
                lock: &*self.0,
            }
        }

        #[inline(always)]
        pub fn write(&self) -> SyncRefWriteGuard<T> {
            self.0.prepare_lock();
            let guard = self.0.lock.write();
            SyncRefWriteGuard {
                guard: ManuallyDrop::new(guard),
                lock: &*self.0,
            }
        }
    }


    struct RwLockDbg<T> {
        lock: RwLock<T>,
        thread_ids: Mutex<Vec<ThreadId>>,
    }

    impl<T> RwLockDbg<T> {
        fn new(value: T) -> RwLockDbg<T> {
            RwLockDbg {
                lock: RwLock::new(value),
                thread_ids: Mutex::new(Vec::new()),
            }
        }

        #[inline(always)]
        fn prepare_lock(&self) {
            let id = std::thread::current().id();
            let mut ids = self.thread_ids.lock();
            if ids.contains(&id) {
                panic!(DEADLOCK_MSG);
            }
            ids.push(id);
        }

        #[inline(always)]
        fn prepare_unlock(&self) {
            let id = std::thread::current().id();
            let mut ids = self.thread_ids.lock();
            ids.remove_item(&id);
        }
    }

    pub struct SyncRefReadGuard<'a, T> {
        guard: ManuallyDrop<RwLockReadGuard<'a, T>>,
        lock: &'a RwLockDbg<T>,
    }

    impl<'a, T> Deref for SyncRefReadGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.guard.deref().deref()
        }
    }

    impl<'a, T> Drop for SyncRefReadGuard<'a, T> {
        fn drop(&mut self) {
            self.lock.prepare_unlock();
            unsafe {
                ManuallyDrop::drop(&mut self.guard);
            }
        }
    }

    pub struct SyncRefWriteGuard<'a, T> {
        guard: ManuallyDrop<RwLockWriteGuard<'a, T>>,
        lock: &'a RwLockDbg<T>,
    }

    impl<'a, T> Deref for SyncRefWriteGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.guard.deref().deref()
        }
    }

    impl<'a, T> DerefMut for SyncRefWriteGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.guard.deref_mut().deref_mut()
        }
    }

    impl<'a, T> Drop for SyncRefWriteGuard<'a, T> {
        fn drop(&mut self) {
            self.lock.prepare_unlock();
            unsafe {
                ManuallyDrop::drop(&mut self.guard);
            }
        }
    }
}

#[cfg(not(debug_assertions))]
mod sync_ref {
    use super::*;

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
}



#[cfg(test)]
mod tests {
    use super::*;

    // This test would result in a deadlock in release mode
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "deadlock detected, lock already acquired in the current thread")]
    fn should_detect_deadlock() {
        let a: SyncRef<String> = SyncRef::new("str".to_string());
        let _b = a.read();
        let _c = a.write();
    }
}