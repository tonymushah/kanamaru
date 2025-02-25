use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{ready, Context, Poll},
};

use prost::Message;
use tauri::{Listener, Runtime};
use tokio_stream::{Stream, StreamExt};

use super::EventListnerStream;

use crate::ipc::{IpcBodyExtractMessageError, IpcMessage, IpcMessageBase};

#[derive(Debug, Clone)]
pub struct EventListnerMessagesStream<L, R, M>
where
    L: Listener<R>,
    R: Runtime,
    M: Message + Default,
{
    inner: EventListnerStream<L, R>,
    ghosty_message_type: PhantomData<M>,
}

impl<L, R, M> From<EventListnerStream<L, R>> for EventListnerMessagesStream<L, R, M>
where
    L: Listener<R>,
    R: Runtime,
    M: Message + Default,
{
    fn from(inner: EventListnerStream<L, R>) -> Self {
        Self {
            inner,
            ghosty_message_type: PhantomData::<M>,
        }
    }
}

impl<L, R> EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    pub fn into_message_stream<M>(self) -> EventListnerMessagesStream<L, R, M>
    where
        M: Message + Default,
    {
        self.into()
    }
}

impl<L, R, M> From<EventListnerMessagesStream<L, R, M>> for EventListnerStream<L, R>
where
    L: Listener<R>,
    R: Runtime,
    M: Message + Default,
{
    fn from(value: EventListnerMessagesStream<L, R, M>) -> Self {
        value.inner
    }
}

impl<L, R, M> EventListnerMessagesStream<L, R, M>
where
    L: Listener<R>,
    R: Runtime,
    M: Message + Default,
{
    pub fn into_inner(self) -> EventListnerStream<L, R> {
        self.into()
    }
    pub fn map_message<M1>(self) -> EventListnerMessagesStream<L, R, M1>
    where
        M1: Message + Default,
    {
        EventListnerMessagesStream {
            inner: self.inner,
            ghosty_message_type: PhantomData::<M1>,
        }
    }
}

impl<L, R, M> Unpin for EventListnerMessagesStream<L, R, M>
where
    L: Listener<R>,
    R: Runtime,
    M: Message + Default,
{
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum EventListnerMessagesStreamParseError {
    Json(#[from] serde_json::Error),
    IpcBodyExtractMessage(#[from] IpcBodyExtractMessageError),
}

impl<L, R, M> Stream for EventListnerMessagesStream<L, R, M>
where
    L: Listener<R>,
    R: Runtime,
    M: Message + Default,
{
    type Item = Result<IpcMessage<M>, EventListnerMessagesStreamParseError>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut inner_next = Box::pin(self.inner.next());

        if let Some(d) = ready!(inner_next.as_mut().poll(cx)) {
            match serde_json::from_str::<Option<IpcMessageBase>>(&d)
                .map_err(EventListnerMessagesStreamParseError::Json)
            {
                Ok(res) => {
                    if let Some(base) = res {
                        let message = <IpcMessage<M> as TryFrom<IpcMessageBase>>::try_from(base);
                        Poll::Ready(Some(message.map_err(
                            EventListnerMessagesStreamParseError::IpcBodyExtractMessage,
                        )))
                    } else {
                        Poll::Ready(None)
                    }
                }
                Err(err) => Poll::Ready(Some(Err(err))),
            }
        } else {
            Poll::Ready(None)
        }
    }
}
