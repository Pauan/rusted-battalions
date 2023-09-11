use std::sync::Arc;
use futures_signals::signal_vec::{MutableVec, SignalVec, MutableVecLockRef};


#[repr(transparent)]
pub struct SortedVec<T> {
    mutable: MutableVec<Arc<T>>,
}

impl<T> SortedVec<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            mutable: MutableVec::new(),
        }
    }

    #[inline]
    pub fn with_values(mut values: Vec<Arc<T>>) -> Self {
        values.sort_unstable_by_key(Arc::as_ptr);

        Self {
            mutable: MutableVec::new_with_values(values),
        }
    }

    #[inline]
    pub fn signal_vec(&self) -> impl SignalVec<Item = Arc<T>> {
        self.mutable.signal_vec_cloned()
    }

    #[inline]
    pub fn lock_ref(&self) -> MutableVecLockRef<'_, Arc<T>> {
        self.mutable.lock_ref()
    }


    pub fn insert(&self, value: Arc<T>) {
        let mut lock = self.mutable.lock_mut();

        match lock.binary_search_by_key(&Arc::as_ptr(&value), Arc::as_ptr) {
            Ok(_) => {
                panic!("Value already exists in SortedVec");
            },
            Err(index) => {
                lock.insert_cloned(index, value);
            },
        }
    }

    pub fn remove(&self, value: &Arc<T>) {
        let mut lock = self.mutable.lock_mut();

        match lock.binary_search_by_key(&Arc::as_ptr(value), Arc::as_ptr) {
            Ok(index) => {
                lock.remove(index);
            },
            Err(_) => {
                panic!("Value doesn't exist in SortedVec");
            },
        }
    }


    /*pub fn insert(&self, value: Arc<T>) {
        let mut lock = self.mutable.lock_mut();

        lock.push_cloned(value);
    }

    pub fn remove(&self, value: &Arc<T>) {
        let mut lock = self.mutable.lock_mut();

        if let Some(index) = lock.iter().position(|x| Arc::ptr_eq(x, value)) {
            lock.remove(index);
        }
    }*/
}
