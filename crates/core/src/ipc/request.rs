pub mod streaming;
pub mod unary;

pub use streaming::StreamingRequest;
pub use unary::UnaryRequest;

use std::{fmt::Debug, sync::Arc};

use serde::Deserialize;
use tauri::{
    ipc::{Channel, InvokeMessage, JavaScriptChannelId},
    AppHandle, Listener, Manager, Runtime, Webview,
};
use tokio_util::sync::CancellationToken;

use crate::{utils::CancellationTokenListener, Status};

use super::{IpcBodyExtractMessageError, IpcMessageBase};

#[derive(Clone, Deserialize)]
pub struct RawRequest {
    pub route: String,
    pub cancel_token_event_id: String,
    pub payload: Option<IpcMessageBase>,
    pub client_streaming_event_id: Option<String>,
    pub server_streaming_event_id: Option<String>,
    pub status_channel: Arc<JavaScriptChannelId>,
}

impl Debug for RawRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct RawRequestDbg<'a> {
            route: &'a String,
            cancel_token_event_id: &'a String,
            payload: Option<&'a IpcMessageBase>,
            client_streaming_event_id: Option<&'a String>,
            server_streaming_event_id: Option<&'a String>,
        }
        RawRequestDbg {
            route: &self.route,
            cancel_token_event_id: &self.cancel_token_event_id,
            payload: self.payload.as_ref(),
            client_streaming_event_id: self.client_streaming_event_id.as_ref(),
            server_streaming_event_id: self.server_streaming_event_id.as_ref(),
        }
        .fmt(f)
    }
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

#[derive(Debug, thiserror::Error)]
pub enum RawRequestToRequestError {
    #[error("Missing payload")]
    MissingPayload,
    #[error(transparent)]
    IpcBodyExtractMessage(#[from] IpcBodyExtractMessageError),
    #[error("The invoke message payload format is invalid which is raw")]
    InvalidPayloadFormat,
    #[error("the client streaming event id is missing in the request")]
    MissingClientStreamingEventId,
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
    fn status_channel(&self) -> Channel<Status>;
}
