use std::sync::Arc;

use prost::Message;
use serde::Deserialize;
use tauri::{
    http::HeaderMap,
    ipc::{InvokeBody, InvokeMessage},
    Runtime, Webview,
};

use crate::utils::CancellationTokenListener;

use super::{InvokeMessageToRequestError, RawRequest, RawRequestToRequestError, RequestBase};

#[derive(Debug, Clone)]
pub struct UnaryRequest<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    token: Arc<CancellationTokenListener<Webview<R>, R>>,
    message: M,
    headers: HeaderMap,
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
}
