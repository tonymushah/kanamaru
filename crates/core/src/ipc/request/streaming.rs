use std::{fmt::Debug, sync::Arc};

use prost::Message;
use serde::Deserialize;
use tauri::{
    http::HeaderMap,
    ipc::{Channel, InvokeBody, InvokeMessage},
    Runtime, Webview,
};

use crate::{
    utils::{
        event_listener_stream::message::EventListnerMessagesStream, CancellationTokenListener,
        EventListnerStream,
    },
    Status,
};

use super::{InvokeMessageToRequestError, RawRequest, RawRequestToRequestError, RequestBase};

pub struct StreamingRequest<R, M>
where
    R: Runtime,
    M: Message + Default,
{
    metadata: HeaderMap,
    channel_status: Channel<Status>,
    stream: EventListnerMessagesStream<Webview<R>, R, M>,
    cancel_token: Arc<CancellationTokenListener<Webview<R>, R>>,
}

impl<R, M> Debug for StreamingRequest<R, M>
where
    R: Runtime,
    M: Message + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SteamingRequest")
            .field("metadata", &self.metadata)
            .field("channel_status", &())
            .field("stream", &())
            .field("cancel_token", &self.cancel_token)
            .finish()
    }
}

impl<R, M> StreamingRequest<R, M>
where
    R: Runtime,
    M: Message + Default,
{
    pub fn new(
        token: Arc<CancellationTokenListener<Webview<R>, R>>,
        raw: RawRequest,
    ) -> Result<Self, RawRequestToRequestError> {
        if let Some(client_stream_id) = raw.client_streaming_event_id {
            Ok(Self {
                channel_status: raw.status_channel.channel_on(token.listener().clone()),
                metadata: raw.payload.map(|e| e.metadata).unwrap_or_default(),
                stream: EventListnerStream::new(token.listener().clone(), client_stream_id)
                    .into_message_stream(),
                cancel_token: token,
            })
        } else {
            Err(RawRequestToRequestError::MissingClientStreamingEventId)
        }
    }
    pub fn headers_ref(&self) -> &HeaderMap {
        &self.metadata
    }
    pub fn headers(&self) -> HeaderMap {
        self.headers_ref().clone()
    }
    pub fn stream(self) -> EventListnerMessagesStream<Webview<R>, R, M> {
        self.stream
    }
    pub fn stream_mut(&mut self) -> &mut EventListnerMessagesStream<Webview<R>, R, M> {
        &mut self.stream
    }
    pub fn map_message<M1>(self) -> StreamingRequest<R, M1>
    where
        M1: Message + Default,
    {
        StreamingRequest {
            metadata: self.metadata,
            channel_status: self.channel_status,
            stream: self.stream.map_message(),
            cancel_token: self.cancel_token,
        }
    }
}

impl<R, M> TryFrom<&InvokeMessage<R>> for StreamingRequest<R, M>
where
    R: Runtime,
    M: Message + Default,
{
    type Error = InvokeMessageToRequestError;
    fn try_from(value: &InvokeMessage<R>) -> Result<Self, Self::Error> {
        if let InvokeBody::Json(payload) = value.payload() {
            let raw_req: RawRequest = Deserialize::deserialize(payload)?;
            let token = raw_req.cancel_token(value.webview());
            Ok(StreamingRequest::new(Arc::new(token), raw_req)?)
        } else {
            Err(InvokeMessageToRequestError::RawRequestToRequest(
                RawRequestToRequestError::InvalidPayloadFormat,
            ))
        }
    }
}

impl<R, M> RequestBase<R> for StreamingRequest<R, M>
where
    R: Runtime,
    M: Message + Default,
{
    fn token(&self) -> Arc<CancellationTokenListener<Webview<R>, R>> {
        self.cancel_token.clone()
    }
    fn status_channel(&self) -> Channel<crate::Status> {
        self.channel_status.clone()
    }
}
