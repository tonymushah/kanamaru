use std::sync::Arc;

use prost::Message;
use serde::Deserialize;
use tauri::{
    http::HeaderMap,
    ipc::{InvokeBody, InvokeMessage},
    Runtime, Webview,
};

use crate::utils::{
    event_listener_stream::message::EventListnerMessagesStream, CancellationTokenListener,
    EventListnerStream,
};

use super::{InvokeMessageToRequestError, RawRequest, RawRequestToRequestError, RequestBase};

#[derive(Debug)]
pub struct StreamingRequest<R, M>
where
    R: Runtime,
    M: Message + Default,
{
    metadata: HeaderMap,
    stream: EventListnerMessagesStream<Webview<R>, R, M>,
    cancel_token: Arc<CancellationTokenListener<Webview<R>, R>>,
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
}
