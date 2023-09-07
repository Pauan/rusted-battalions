use wasm_bindgen_futures::spawn_local;
use futures_signals::signal::{Mutable, SignalExt};
use futures::future::{BoxFuture, AbortHandle, abortable};
use futures::stream::{StreamExt};
use futures::channel::mpsc::{UnboundedSender, UnboundedReceiver, unbounded};
use std::future::Future;


pub struct FutureSpawner {
    // TODO figure out a better way to wait until the game engine is warmed up
    started: Mutable<bool>,
    // TODO figure out a way to get rid of the mpsc
    sender: UnboundedSender<BoxFuture<'static, ()>>,
    handle: AbortHandle,
}

impl FutureSpawner {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();

        let started = Mutable::new(false);

        let wait_for = started.signal().wait_for(true);

        let (future, handle) = abortable(async move {
            wait_for.await;
            receiver.for_each_concurrent(None, move |future| future).await;
        });

        spawn_local(async move {
            let _ = future.await;
        });

        Self {
            started,
            sender,
            handle,
        }
    }

    pub fn start(&self) {
        self.started.set_neq(true);
    }

    #[inline]
    pub fn spawn<F>(&self, future: F) where F: Future<Output = ()> + Send + 'static {
        self.sender.unbounded_send(Box::pin(future)).unwrap();
    }
}

impl Drop for FutureSpawner {
    #[inline]
    fn drop(&mut self) {
        self.handle.abort();
    }
}
