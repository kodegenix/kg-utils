use std::sync::Arc;
use std::ops::{Deref, DerefMut};


use parking_lot::{
    RwLock,
    RwLockReadGuard, RwLockWriteGuard,
    MappedRwLockReadGuard, MappedRwLockWriteGuard,
};

const DEADLOCK_MSG: &str = "deadlock detected, lock already acquired in the current thread";


pub use sync_ref::{SyncRef, SyncRefReadGuard, SyncRefWriteGuard, SyncRefMapReadGuard, SyncRefMapWriteGuard};

unsafe impl<T> Send for SyncRef<T> {}

unsafe impl<T> Sync for SyncRef<T> {}

impl<T> PartialEq for SyncRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for SyncRef<T> {}

#[cfg(debug_assertions)]
mod sync_ref {
    use super::*;
    use std::ptr::NonNull;
    use std::thread::ThreadId;
    use parking_lot::Mutex;

    pub struct SyncRef<T>(pub(super) Arc<RwLockDbg<T>>);

    impl<T> SyncRef<T> {
        pub fn new(value: T) -> SyncRef<T> {
            SyncRef(Arc::new(RwLockDbg::new(value)))
        }

        #[inline(always)]
        pub fn read(&self) -> SyncRefReadGuard<T> {
            self.0.threads.check_current_thread();
            let guard = self.0.lock.read();
            SyncRefReadGuard {
                guard,
                threads: ThreadsPtr::new(&self.0.threads),
            }
        }

        #[inline(always)]
        pub fn write(&self) -> SyncRefWriteGuard<T> {
            self.0.threads.check_current_thread();
            let guard = self.0.lock.write();
            SyncRefWriteGuard {
                guard,
                threads: ThreadsPtr::new(&self.0.threads),
            }
        }
    }


    impl<T> Clone for SyncRef<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    struct Threads(Mutex<Vec<ThreadId>>);

    impl Threads {
        #[inline(always)]
        fn check_current_thread(&self) {
            let id = std::thread::current().id();
            let mut ids = self.0.lock();
            if ids.contains(&id) {
                panic!(DEADLOCK_MSG);
            }
            ids.push(id);
        }

        #[inline(always)]
        fn remove_current_thread(&self) {
            let id = std::thread::current().id();
            let mut ids = self.0.lock();
            ids.remove_item(&id);
        }
    }

    struct ThreadsPtr(NonNull<Threads>);

    impl ThreadsPtr {
        fn new(threads: &Threads) -> ThreadsPtr {
            ThreadsPtr(NonNull::from(threads))
        }
    }

    impl Drop for ThreadsPtr {
        fn drop(&mut self) {
            unsafe { self.0.as_mut() }.remove_current_thread();
        }
    }

    pub(super) struct RwLockDbg<T> {
        lock: RwLock<T>,
        threads: Threads,
    }

    impl<T> RwLockDbg<T> {
        fn new(value: T) -> RwLockDbg<T> {
            RwLockDbg {
                lock: RwLock::new(value),
                threads: Threads(Mutex::new(Vec::new())),
            }
        }
    }

    pub struct SyncRefReadGuard<'a, T> {
        threads: ThreadsPtr,
        guard: RwLockReadGuard<'a, T>,
    }

    impl<'a, T> SyncRefReadGuard<'a, T> {
        pub fn map<U, F>(s: Self, f: F) -> SyncRefMapReadGuard<'a, U>
            where F: FnOnce(&T) -> &U
        {
            let threads = s.threads;
            let guard = s.guard;
            SyncRefMapReadGuard {
                threads,
                guard: RwLockReadGuard::map(guard, f),
            }
        }
    }

    impl<'a, T> Deref for SyncRefReadGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.guard.deref()
        }
    }

    pub struct SyncRefWriteGuard<'a, T> {
        threads: ThreadsPtr,
        guard: RwLockWriteGuard<'a, T>,
    }

    impl<'a, T> SyncRefWriteGuard<'a, T> {
        pub fn map<U, F>(s: Self, f: F) -> SyncRefMapWriteGuard<'a, U>
            where F: FnOnce(&mut T) -> &mut U
        {
            let threads = s.threads;
            let guard = s.guard;
            SyncRefMapWriteGuard {
                threads,
                guard: RwLockWriteGuard::map(guard, f),
            }
        }
    }

    impl<'a, T> Deref for SyncRefWriteGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.guard.deref()
        }
    }

    impl<'a, T> DerefMut for SyncRefWriteGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.guard.deref_mut()
        }
    }


    pub struct SyncRefMapReadGuard<'a, T> {
        #[allow(dead_code)]
        threads: ThreadsPtr,
        guard: MappedRwLockReadGuard<'a, T>,
    }

    impl<'a, T> Deref for SyncRefMapReadGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.guard.deref()
        }
    }

    pub struct SyncRefMapWriteGuard<'a, T> {
        #[allow(dead_code)]
        threads: ThreadsPtr,
        guard: MappedRwLockWriteGuard<'a, T>,
    }

    impl<'a, T> Deref for SyncRefMapWriteGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.guard.deref()
        }
    }

    impl<'a, T> DerefMut for SyncRefMapWriteGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.guard.deref_mut()
        }
    }
}

#[cfg(not(debug_assertions))]
mod sync_ref {
    use super::*;

    pub struct SyncRef<T>(pub(super) Arc<RwLock<T>>);

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

    impl<T> Clone for SyncRef<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    pub struct SyncRefReadGuard<'a, T>(RwLockReadGuard<'a, T>);

    impl<'a, T> SyncRefReadGuard<'a, T> {
        pub fn map<U, F>(s: Self, f: F) -> SyncRefMapReadGuard<'a, U>
            where F: FnOnce(&T) -> &U
        {
            let guard = s.0;
            SyncRefMapReadGuard(RwLockReadGuard::map(guard, f))
        }
    }

    impl<'a, T> Deref for SyncRefReadGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.0.deref()
        }
    }


    pub struct SyncRefWriteGuard<'a, T>(RwLockWriteGuard<'a, T>);

    impl<'a, T> SyncRefWriteGuard<'a, T> {
        pub fn map<U, F>(s: Self, f: F) -> SyncRefMapWriteGuard<'a, U>
            where F: FnOnce(&mut T) -> &mut U
        {
            let guard = s.0;
            SyncRefMapWriteGuard(RwLockWriteGuard::map(guard, f))
        }
    }

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

    pub struct SyncRefMapReadGuard<'a, T>(MappedRwLockReadGuard<'a, T>);

    impl<'a, T> Deref for SyncRefMapReadGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.0.deref()
        }
    }


    pub struct SyncRefMapWriteGuard<'a, T>(MappedRwLockWriteGuard<'a, T>);

    impl<'a, T> Deref for SyncRefMapWriteGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.0.deref()
        }
    }

    impl<'a, T> DerefMut for SyncRefMapWriteGuard<'a, T> {
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
        let a: SyncRef<()> = SyncRef::new(());
        let _b = a.read();
        let _c = a.write();
    }
}