use rusted_battalions_engine::Spawner;
use std::future::Future;
use std::task::{Context, Waker};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::pin::Pin;
use futures::task::{ArcWake, waker};


struct TaskInner {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

/// SAFETY: This is safe because the code guarantees that the `future` is
///         only ever mutably accessed on the thread which spawned it.
unsafe impl Send for TaskInner {}

struct Task {
    inner: Mutex<Option<TaskInner>>,
    pending: AtomicBool,
    executor: Arc<Executor>,
}

impl Task {
    fn spawn(executor: Arc<Executor>, future: Pin<Box<dyn Future<Output = ()>>>) {
        let task = Arc::new(Task {
            inner: Mutex::new(None),
            pending: AtomicBool::new(false),
            executor,
        });

        let inner = TaskInner {
            future,
            waker: waker(task.clone()),
        };

        *task.inner.lock().unwrap() = Some(inner);

        task.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if !self.pending.swap(true, Ordering::SeqCst) {
            let mut pending = self.executor.pending.lock().unwrap();
            pending.push(self.clone());
        }
    }

    // SAFETY: This must only be run on the thread which called `Task::spawn`.
    unsafe fn run(&self) {
        if self.pending.swap(false, Ordering::SeqCst) {
            let mut inner = self.inner.lock().unwrap();

            if let Some(TaskInner { future, waker }) = &mut *inner {
                let poll = {
                    let mut cx = Context::from_waker(waker);
                    future.as_mut().poll(&mut cx)
                };

                // Cleanup Future / Waker immediately
                if poll.is_ready() {
                    *inner = None;
                }
            }
        }
    }
}

impl ArcWake for Task {
    #[inline]
    fn wake_by_ref(this: &Arc<Self>) {
        this.wake_by_ref();
    }
}


struct Executor {
    // Pending Futures which need to be polled
    pending: Mutex<Vec<Arc<Task>>>,

    // Futures which are currently being polled
    iterating: Mutex<Vec<Arc<Task>>>,
}

impl Executor {
    #[inline]
    fn with_capacity(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            pending: Mutex::new(Vec::with_capacity(capacity)),
            iterating: Mutex::new(Vec::with_capacity(capacity)),
        })
    }

    #[inline]
    fn spawn(self: &Arc<Executor>, future: Pin<Box<dyn Future<Output = ()>>>) {
        Task::spawn(self.clone(), future);
    }

    fn run(&self) {
        let mut iterating = self.iterating.lock().unwrap();

        assert_eq!(iterating.len(), 0);

        {
            let mut pending = self.pending.lock().unwrap();

            // If there are no pending futures then do nothing.
            if pending.is_empty() {
                return;
            }

            // we need to poll every future inside of pending,
            // however because we need exclusive mutable access
            // to the Vec, that means we have to temporarily swap
            // the pending and iterating Vec, and then swap them
            // back when we're done.
            std::mem::swap(&mut *iterating, &mut *pending);
        }

        // This will keep processing the pending futures until
        // there aren't any more pending futures.
        loop {
            // Poll the futures.
            for task in iterating.drain(..) {
                // SAFETY: This is safe because we know that the
                //         Task was spawned by this Executor,
                //         which means the Future is being polled
                //         on the same thread where it was created.
                unsafe {
                    task.run();
                }
            }

            {
                let mut pending = self.pending.lock().unwrap();

                // Swap back pending and iterating
                std::mem::swap(&mut *iterating, &mut *pending);

                // No more pending futures, work is done.
                if iterating.is_empty() {
                    break;
                }
            }
        }
    }
}


thread_local! {
    // TODO figure out best initial capacity
    static FUTURES: Arc<Executor> = Executor::with_capacity(512);
}

pub fn spawn_local(future: Pin<Box<dyn Future<Output = ()>>>) {
    FUTURES.with(|futures| futures.spawn(future))
}

pub fn run_futures() {
    FUTURES.with(|futures| futures.run())
}


pub struct CustomSpawner;

impl Spawner for CustomSpawner {
    #[inline]
    fn spawn_local(&self, future: Pin<Box<dyn Future<Output = ()>>>) {
        spawn_local(future);
    }
}
