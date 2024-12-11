use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    task::{ready, Poll},
};

use tauri::{EventId, Listener, Runtime};
use tokio::sync::watch::{self, Receiver};
use tokio_stream::Stream;

#[derive(Debug)]
pub struct EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    listener: L,
    runtime: PhantomData<R>,
    event_id: EventId,
    receiver: Receiver<Option<String>>,
    label: String,
}

impl<L, R> Drop for EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    fn drop(&mut self) {
        self.listener.unlisten(self.event_id);
    }
}

impl<L, R> EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    pub fn new<E: Into<String>>(listener: L, event: E) -> Self {
        let label = event.into();
        let (tx, rx) = watch::channel(None::<String>);
        let event_id = listener.listen(&label, move |event| {
            tx.send_replace(Some(event.payload().into()));
        });
        Self {
            listener,
            runtime: PhantomData::<R>,
            event_id,
            receiver: rx,
            label,
        }
    }
    pub fn listener(&self) -> &L {
        &self.listener
    }
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl<L, R> Clone for EventListnerStream<L, R>
where
    L: Listener<R> + Clone,
    R: Runtime,
{
    fn clone(&self) -> Self {
        Self::new(self.listener.clone(), self.label())
    }
}
impl<L, R> Unpin for EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
}

impl<L, R> Stream for EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    type Item = String;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut fut = Box::pin(self.receiver.changed());
        if ready!(fut.as_mut().poll(cx)).is_ok() {
            drop(fut);
            if let Some(data) = self.receiver.borrow().as_ref() {
                Poll::Ready(Some(data.clone()))
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
}
