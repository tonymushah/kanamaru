use std::marker::PhantomData;

use prost::Message;
use tauri::{Emitter, EventTarget, Runtime, Webview};
use tokio_stream::{Stream, StreamExt};

use crate::ipc::{IpcMessage, IpcMessageBase};

#[derive(Debug)]
pub struct StreamingResponse<M, S> {
    phantom: PhantomData<M>,
    stream: S,
}

impl<M: Message, S: Stream<Item = Result<IpcMessage<M>, crate::Status>>> StreamingResponse<M, S> {
    pub fn new(stream: S) -> Self {
        Self {
            phantom: PhantomData::<M>,
            stream,
        }
    }
    pub async fn send_responses<R: Runtime>(
        self,
        webview: Webview<R>,
        event_stream_id: String,
    ) -> tauri::Result<()> {
        let mut stream = Box::pin(self.stream);
        while let Some(res) = stream.next().await {
            webview.emit_to(
                EventTarget::webview(webview.label()),
                &event_stream_id,
                res.map(Into::<IpcMessageBase>::into),
            )?;
        }
        webview.emit_to(
            EventTarget::webview(webview.label()),
            &event_stream_id,
            None::<()>,
        )?;
        Ok(())
    }
}
