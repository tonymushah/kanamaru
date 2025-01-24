use prost::Message;
use tauri::http::HeaderMap;

use crate::ipc::IpcMessage;

#[derive(Debug, Clone, Default)]
pub struct UnaryResponse<M: Message> {
    pub body: M,
    pub metadata: HeaderMap,
}

impl<M: Message> UnaryResponse<M> {
    pub fn new(body: M) -> Self {
        Self {
            body,
            metadata: Default::default(),
        }
    }
    pub fn new_with_metadata(body: M, metadata: HeaderMap) -> Self {
        Self { body, metadata }
    }
}

impl<M: Message> From<UnaryResponse<M>> for IpcMessage<M> {
    fn from(value: UnaryResponse<M>) -> Self {
        Self {
            body: value.body,
            metadata: value.metadata,
        }
    }
}
