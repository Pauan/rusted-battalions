use std::ops::DerefMut;
use crate::scene::{NodeHandle, NodeLayout};

pub(crate) mod builders;
pub(crate) mod buffer;
pub(crate) mod macros;
pub(crate) mod unicode;


pub(crate) trait IsAtomic {
    type Cell;

    fn new(v: Self) -> Self::Cell;
    fn get(cell: &Self::Cell) -> Self;
    fn set(cell: &Self::Cell, val: Self);
    fn replace(cell: &Self::Cell, val: Self) -> Self;
}


#[cfg(feature = "thread-safe")]
mod sync {
    use std::ops::DerefMut;
    use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};


    pub(crate) type Arc<T> = std::sync::Arc<T>;


    #[repr(transparent)]
    pub(crate) struct AtomicF32 {
        cell: AtomicU32,
    }

    impl AtomicF32 {
        #[inline]
        fn new(v: f32) -> Self {
            Self {
                cell: AtomicU32::new(v.to_bits()),
            }
        }

        #[inline]
        fn load(&self, order: Ordering) -> f32 {
            f32::from_bits(self.cell.load(order))
        }

        #[inline]
        fn store(&self, val: f32, order: Ordering) {
            self.cell.store(val.to_bits(), order)
        }

        #[inline]
        fn swap(&self, val: f32, order: Ordering) -> f32 {
            f32::from_bits(self.cell.swap(val.to_bits(), order))
        }
    }


    #[repr(transparent)]
    pub(crate) struct AtomicF64 {
        cell: AtomicU64,
    }

    impl AtomicF64 {
        #[inline]
        fn new(v: f64) -> Self {
            Self {
                cell: AtomicU64::new(v.to_bits()),
            }
        }

        #[inline]
        fn load(&self, order: Ordering) -> f64 {
            f64::from_bits(self.cell.load(order))
        }

        #[inline]
        fn store(&self, val: f64, order: Ordering) {
            self.cell.store(val.to_bits(), order)
        }

        #[inline]
        fn swap(&self, val: f64, order: Ordering) -> f64 {
            f64::from_bits(self.cell.swap(val.to_bits(), order))
        }
    }


    macro_rules! is_atomic {
        ($($from:ty => $to:ty)*) => {
            $(impl super::IsAtomic for $from {
                type Cell = $to;

                #[inline]
                fn new(v: Self) -> Self::Cell {
                    Self::Cell::new(v)
                }

                #[inline]
                fn get(cell: &Self::Cell) -> Self {
                    cell.load(Ordering::SeqCst)
                }

                #[inline]
                fn set(cell: &Self::Cell, val: Self) {
                    cell.store(val, Ordering::SeqCst)
                }

                #[inline]
                fn replace(cell: &Self::Cell, val: Self) -> Self {
                    cell.swap(val, Ordering::SeqCst)
                }
            })*
        };
    }

    is_atomic! {
        bool => std::sync::atomic::AtomicBool
        i8 => std::sync::atomic::AtomicI8
        i16 => std::sync::atomic::AtomicI16
        i32 => std::sync::atomic::AtomicI32
        i64 => std::sync::atomic::AtomicI64
        isize => std::sync::atomic::AtomicIsize
        u8 => std::sync::atomic::AtomicU8
        u16 => std::sync::atomic::AtomicU16
        u32 => std::sync::atomic::AtomicU32
        u64 => std::sync::atomic::AtomicU64
        usize => std::sync::atomic::AtomicUsize
        f32 => AtomicF32
        f64 => AtomicF64
    }


    #[repr(transparent)]
    pub(crate) struct Mutex<T: ?Sized> {
        cell: std::sync::Mutex<T>,
    }

    impl<T> Mutex<T> {
        #[inline]
        pub(crate) fn new(value: T) -> Self {
            Self {
                cell: std::sync::Mutex::new(value),
            }
        }
    }

    impl<T: ?Sized> Mutex<T> {
        #[inline]
        pub(crate) fn lock(&self) -> impl DerefMut<Target = T> + '_ {
            self.cell.lock().unwrap()
        }
    }
}


#[cfg(not(feature = "thread-safe"))]
mod sync {
    use std::ops::DerefMut;
    use std::rc::Rc;
    use std::cell::{Cell, RefCell};


    pub(crate) type Arc<T> = Rc<T>;


    macro_rules! is_atomic {
        ($($from:ty => $to:ty)*) => {
            $(impl super::IsAtomic for $from {
                type Cell = $to;

                #[inline]
                fn new(v: Self) -> Self::Cell {
                    Self::Cell::new(v)
                }

                #[inline]
                fn get(cell: &Self::Cell) -> Self {
                    cell.get()
                }

                #[inline]
                fn set(cell: &Self::Cell, val: Self) {
                    cell.set(val)
                }

                #[inline]
                fn replace(cell: &Self::Cell, val: Self) -> Self {
                    cell.replace(val)
                }
            })*
        };
    }

    is_atomic! {
        bool => Cell<bool>
        i8 => Cell<i8>
        i16 => Cell<i16>
        i32 => Cell<i32>
        i64 => Cell<i64>
        isize => Cell<isize>
        u8 => Cell<u8>
        u16 => Cell<u16>
        u32 => Cell<u32>
        u64 => Cell<u64>
        usize => Cell<usize>
        f32 => Cell<f32>
        f64 => Cell<f64>
    }


    #[repr(transparent)]
    pub(crate) struct Mutex<T: ?Sized> {
        cell: RefCell<T>,
    }

    impl<T> Mutex<T> {
        #[inline]
        pub(crate) fn new(value: T) -> Self {
            Self {
                cell: RefCell::new(value),
            }
        }
    }

    impl<T: ?Sized> Mutex<T> {
        #[inline]
        pub(crate) fn lock(&self) -> impl DerefMut<Target = T> + '_ {
            self.cell.borrow_mut()
        }
    }
}


pub(crate) use sync::Arc;
use sync::Mutex;


#[repr(transparent)]
pub(crate) struct Atomic<T> where T: IsAtomic {
    cell: T::Cell,
}

impl<T> Atomic<T> where T: IsAtomic {
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Self {
            cell: T::new(value),
        }
    }

    #[inline]
    pub(crate) fn get(&self) -> T {
        T::get(&self.cell)
    }

    #[inline]
    pub(crate) fn set(&self, value: T) {
        T::set(&self.cell, value)
    }

    #[inline]
    pub(crate) fn replace(&self, value: T) -> T {
        T::replace(&self.cell, value)
    }
}


pub(crate) struct Lock<T: ?Sized> {
    state: Arc<Mutex<T>>,
}

impl<T> Lock<T> {
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Self {
            state: Arc::new(Mutex::new(value)),
        }
    }
}

impl<T> Lock<T> where T: ?Sized {
    #[inline]
    pub(crate) fn lock(&self) -> impl DerefMut<Target = T> + '_ {
        self.state.lock()
    }
}

impl<T> Lock<T> where T: NodeLayout + 'static {
    #[inline]
    pub(crate) fn into_handle(self) -> NodeHandle {
        NodeHandle {
            layout: Lock {
                state: self.state,
            },
        }
    }
}

impl<T> Clone for Lock<T> where T: ?Sized {
    #[inline]
    fn clone(&self) -> Self {
        Self { state: self.state.clone() }
    }
}
