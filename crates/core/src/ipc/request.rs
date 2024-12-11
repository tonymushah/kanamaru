use std::sync::Arc;

use prost::Message;
use serde::Deserialize;
use tauri::{
    http::HeaderMap,
    ipc::{InvokeBody, InvokeMessage},
    AppHandle, Listener, Manager, Runtime, Webview,
};
use tokio_util::sync::CancellationToken;

use crate::utils::CancellationTokenListener;

use super::{IpcBody, IpcBodyExtractMessageError};

#[derive(Debug, Clone, Deserialize)]
pub struct RawRequest {
    pub cancel_token_event_id: String,
    pub payload: Option<IpcBody>,
    pub client_streaming_event_id: Option<String>,
    pub server_streaming_evente_id: Option<String>,
}

impl RawRequest {
    pub fn cancel_token<L, R>(&self, listener: L) -> CancellationTokenListener<L, R>
    where
        L: Listener<R>,
        R: Runtime,
    {
        CancellationTokenListener::new(listener, &self.cancel_token_event_id)
    }
}

#[derive(Debug, Clone)]
pub struct Request<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    token: Arc<CancellationTokenListener<Webview<R>, R>>,
    message: M,
    headers: HeaderMap,
}

#[derive(Debug, thiserror::Error)]
pub enum RawRequestToRequestError {
    #[error("Missing payload")]
    MissingPayload,
    #[error(transparent)]
    IpcBodyExtractMessage(#[from] IpcBodyExtractMessageError),
    #[error("The invoke message payload format is invalid which is raw")]
    InvalidPayloadFormat,
}

impl<R, M> Request<R, M>
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

impl<R, M> AsRef<M> for Request<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    fn as_ref(&self) -> &M {
        &self.message
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum InvokeMessageToRequestError {
    RawRequestToRequest(#[from] RawRequestToRequestError),
    Json(#[from] serde_json::Error),
}

pub trait RequestBase<R: Runtime>:
    for<'a> TryFrom<&'a InvokeMessage<R>, Error = InvokeMessageToRequestError>
{
    fn token(&self) -> Arc<CancellationTokenListener<Webview<R>, R>>;
    fn cancel_token(&self) -> CancellationToken {
        self.token().token()
    }
    fn webview(&self) -> Webview<R> {
        self.token().listener().clone()
    }
    fn app_handle(&self) -> AppHandle<R> {
        self.webview().app_handle().clone()
    }
}

impl<M, R> TryFrom<&InvokeMessage<R>> for Request<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    type Error = InvokeMessageToRequestError;
    fn try_from(value: &InvokeMessage<R>) -> Result<Self, Self::Error> {
        if let InvokeBody::Json(payload) = value.payload() {
            let raw_req: RawRequest = Deserialize::deserialize(payload)?;
            let token = raw_req.cancel_token(value.webview());
            Ok(Request::new(Arc::new(token), raw_req)?)
        } else {
            Err(InvokeMessageToRequestError::RawRequestToRequest(
                RawRequestToRequestError::InvalidPayloadFormat,
            ))
        }
    }
}

impl<M, R> RequestBase<R> for Request<R, M>
where
    M: Message + Clone + Default,
    R: Runtime,
{
    fn token(&self) -> Arc<CancellationTokenListener<Webview<R>, R>> {
        self.token.clone()
    }
}
