use super::*;

use std::sync::Arc;
use std::cell::{RefCell, Ref, RefMut};
use std::ops::{Deref, DerefMut};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard, MappedReentrantMutexGuard};
use std::borrow::Borrow;


pub struct SyncRef<T>(Arc<ReentrantMutex<RefCell<T>>>);

impl<T> SyncRef<T> {
    pub fn new(value: T) -> SyncRef<T> {
        SyncRef(Arc::new(ReentrantMutex::new(RefCell::new(value))))
    }

    pub fn read<'a>(&'a self) -> SyncRefReadGuard<'a, T> {
        let g = self.0.lock();
        let r = unsafe { std::mem::transmute::<Ref<'_, _>, Ref<'a, _>>(g.deref().borrow()) };
        SyncRefReadGuard { r, g }
    }

    pub fn write<'a>(&'a self) -> SyncRefWriteGuard<'a, T> {
        let g = self.0.lock();
        let r = unsafe { std::mem::transmute::<RefMut<'_, _>, RefMut<'a, _>>(g.deref().borrow_mut()) };
        SyncRefWriteGuard { r, g }
    }
}

unsafe impl<T> Send for SyncRef<T> {}

unsafe impl<T> Sync for SyncRef<T> {}


pub struct SyncRefReadGuard<'a, T> {
    r: Ref<'a, T>,
    g: ReentrantMutexGuard<'a, RefCell<T>>,
}

impl<'a, T> Deref for SyncRefReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.r.deref()
    }
}


pub struct SyncRefWriteGuard<'a, T> {
    r: RefMut<'a, T>,
    g: ReentrantMutexGuard<'a, RefCell<T>>,
}

impl<'a, T> Deref for SyncRefWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.r.deref()
    }
}

impl<'a, T> DerefMut for SyncRefWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.r.deref_mut()
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