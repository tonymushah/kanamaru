#[doc(hidden)]
pub mod codegen;
pub mod ipc;
mod macros;
pub mod plugin;
pub mod responder;
pub mod status;
pub mod utils;

pub use async_trait::async_trait;

pub use ipc::{
    request::RequestBase, StreamingRequest, StreamingResponse, UnaryRequest, UnaryResponse,
};
pub use plugin::{builder::Builder, KanamaruPlugin};
pub use responder::{RPCType, Responder, Routes};
pub use status::{AsCode, Code, Status};

pub mod prelude {
    pub use async_trait::async_trait;

    pub use super::ipc::{
        request::RequestBase, StreamingRequest, StreamingResponse, UnaryRequest, UnaryResponse,
    };
    pub use super::plugin::{builder::Builder, KanamaruPlugin};
    pub use super::responder::{RPCType, Responder, Routes};
    pub use super::status::{AsCode, Code, Status};
}
