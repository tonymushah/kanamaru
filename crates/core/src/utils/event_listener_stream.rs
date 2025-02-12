// TODO Use `tokio_util` Reusable Box for future cause the stream will hang on forever
pub mod message;

use std::marker::PhantomData;

use tauri::{EventId, Listener, Runtime};
use tokio::sync::watch::{self};
use tokio_stream::{wrappers::WatchStream, Stream, StreamExt};

#[derive(Debug)]
pub struct EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    listener: L,
    runtime: PhantomData<R>,
    event_id: EventId,
    label: String,
    stream: WatchStream<Option<String>>,
}

unsafe impl<L, R> Send for EventListnerStream<L, R>
where
    L: Listener<R> + Send,
    R: Runtime,
{
}

unsafe impl<L, R> Sync for EventListnerStream<L, R>
where
    L: Listener<R> + Sync,
    R: Runtime,
{
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
            stream: WatchStream::new(rx.clone()),
            listener,
            runtime: PhantomData::<R>,
            event_id,
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
        let mut inner = (&mut self.stream).filter_map(|v| v);
        let mut stream = Box::pin(&mut inner);
        stream.as_mut().poll_next(cx)
    }
}
