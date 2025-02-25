use std::{fmt::Debug, sync::Arc};

use prost::Message;
use serde::Deserialize;
use tauri::{
    http::HeaderMap,
    ipc::{Channel, InvokeBody, InvokeMessage},
    Runtime, Webview,
};

use crate::{utils::CancellationTokenListener, Status};

use super::{InvokeMessageToRequestError, RawRequest, RawRequestToRequestError, RequestBase};

#[derive(Clone)]
pub struct UnaryRequest<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    token: Arc<CancellationTokenListener<Webview<R>, R>>,
    message: M,
    channel_status: Channel<Status>,
    headers: HeaderMap,
}

impl<R: Runtime, M: Message + Clone + Default> Debug for UnaryRequest<R, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnaryRequest")
            .field("token", &self.token)
            .field("message", &self.message)
            .field("channel_status", &())
            .field("headers", &self.headers)
            .finish()
    }
}

impl<R, M> UnaryRequest<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    pub fn new(
        token: Arc<CancellationTokenListener<Webview<R>, R>>,
        raw_reqwest: RawRequest,
    ) -> Result<Self, RawRequestToRequestError> {
        if let Some(payload) = raw_reqwest.payload {
            Ok(Self {
                channel_status: raw_reqwest
                    .status_channel
                    .channel_on(token.listener().clone()),
                token,
                message: payload.extract_message()?,
                headers: payload.metadata,
            })
        } else {
            Err(RawRequestToRequestError::MissingPayload)
        }
    }

    pub fn into_inner(self) -> M {
        self.message
    }
    pub fn message(&self) -> &M {
        self.as_ref()
    }
    pub fn headers_ref(&self) -> &HeaderMap {
        &self.headers
    }
    pub fn headers(&self) -> HeaderMap {
        self.headers_ref().clone()
    }
    pub fn map_message<F, M1>(self, map_fn: F) -> UnaryRequest<R, M1>
    where
        F: FnOnce(M) -> M1,
        M1: Message + Clone + Default,
    {
        UnaryRequest {
            token: self.token,
            message: map_fn(self.message),
            channel_status: self.channel_status,
            headers: self.headers,
        }
    }
    pub fn change_message<F, M1>(self, message: M1) -> UnaryRequest<R, M1>
    where
        F: FnOnce(M) -> M1,
        M1: Message + Clone + Default,
    {
        UnaryRequest {
            token: self.token,
            message,
            channel_status: self.channel_status,
            headers: self.headers,
        }
    }
}

impl<R, M> AsRef<M> for UnaryRequest<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    fn as_ref(&self) -> &M {
        &self.message
    }
}

impl<M, R> TryFrom<&InvokeMessage<R>> for UnaryRequest<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    type Error = InvokeMessageToRequestError;
    fn try_from(value: &InvokeMessage<R>) -> Result<Self, Self::Error> {
        if let InvokeBody::Json(payload) = value.payload() {
            let raw_req: RawRequest = Deserialize::deserialize(payload)?;
            let token = raw_req.cancel_token(value.webview());
            Ok(UnaryRequest::new(Arc::new(token), raw_req)?)
        } else {
            Err(InvokeMessageToRequestError::RawRequestToRequest(
                RawRequestToRequestError::InvalidPayloadFormat,
            ))
        }
    }
}

impl<M, R> RequestBase<R> for UnaryRequest<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    fn token(&self) -> Arc<CancellationTokenListener<Webview<R>, R>> {
        self.token.clone()
    }
    fn status_channel(&self) -> tauri::ipc::Channel<crate::Status> {
        self.channel_status.clone()
    }
}
