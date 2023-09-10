use rusted_battalions_engine::Spawner;
use futures::future::{AbortHandle, Abortable};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::future::Future;
use std::task::{Waker, Poll, Context};
use std::pin::Pin;


// TODO impl Drop ?
struct StartedFuture {
    index: usize,
    state: Arc<StartedState>,
}

impl Future for StartedFuture {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.state.has_started() {
            Poll::Ready(())

        } else {
            self.state.set_waker(self.index, cx.waker().clone());
            Poll::Pending
        }
    }
}


struct StartedState {
    started: AtomicBool,
    wakers: Mutex<Vec<Option<Waker>>>,
}

impl StartedState {
    fn has_started(&self) -> bool {
        self.started.load(Ordering::SeqCst)
    }

    fn wait_for_start(self: &Arc<Self>) -> impl Future<Output = ()> {
        if self.has_started() {
            StartedFuture {
                index: 0,
                state: self.clone(),
            }

        } else {
            let index;

            {
                let mut lock = self.wakers.lock().unwrap();
                index = lock.len();
                lock.push(None);
            }

            StartedFuture {
                index,
                state: self.clone(),
            }
        }
    }

    fn set_waker(&self, index: usize, waker: Waker) {
        let mut lock = self.wakers.lock().unwrap();

        lock[index] = Some(waker);
    }

    fn start(&self) {
        if !self.started.swap(true, Ordering::SeqCst) {
            let mut lock = self.wakers.lock().unwrap();

            for waker in lock.drain(..) {
                if let Some(waker) = waker {
                    waker.wake();
                }
            }

            *lock = vec![];
        }
    }
}


struct Started {
    state: Arc<StartedState>,
}

impl Started {
    #[inline]
    fn new() -> Self {
        Self {
            state: Arc::new(StartedState {
                started: AtomicBool::new(false),
                wakers: Mutex::new(vec![]),
            }),
        }
    }

    #[inline]
    fn wait_for_start(&self) -> impl Future<Output = ()> {
        self.state.wait_for_start()
    }

    #[inline]
    fn start(&self) {
        self.state.start();
    }
}


pub struct FutureSpawner {
    started: Started,
    handles: Mutex<Vec<AbortHandle>>,
}

impl FutureSpawner {
    pub fn new() -> Self {
        Self {
            started: Started::new(),
            handles: Mutex::new(vec![]),
        }
    }

    #[inline]
    pub fn start(&self) {
        self.started.start();
    }

    pub fn cleanup(&self) {
        let mut lock = self.handles.lock().unwrap();

        lock.retain(move |handle| {
            !handle.is_aborted()
        });
    }

    fn push_handle(&self, handle: AbortHandle) {
        self.handles.lock().unwrap().push(handle);
    }

    pub fn spawn<S, F>(&self, spawner: &S, future: F)
        where S: Spawner,
              F: Future<Output = ()> + 'static {

        let wait_for = self.started.wait_for_start();

        let (handle, registration) = AbortHandle::new_pair();

        self.push_handle(handle.clone());

        let future = Abortable::new(async move {
            wait_for.await;
            future.await;

            // This allows the AbortHandle to be cleaned up by the `cleanup()` method.
            // TOOD test this
            handle.abort();
        }, registration);

        spawner.spawn_local(Box::pin(async move {
            let _ = future.await;
        }));
    }

    #[inline]
    pub fn spawn_iter<S, I>(&self, spawner: &S, futures: I)
        where S: Spawner,
              I: IntoIterator,
              I::Item: Future<Output = ()> + 'static {

        for future in futures {
            self.spawn(spawner, future);
        }
    }
}

impl Drop for FutureSpawner {
    fn drop(&mut self) {
        let lock = self.handles.get_mut().unwrap();

        for handle in lock.iter() {
            handle.abort();
        }
    }
}
