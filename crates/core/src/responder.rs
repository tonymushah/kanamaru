use std::{collections::HashMap, sync::Arc};

use tauri::{ipc::InvokeResolver, Runtime, Webview};

use crate::{
    ipc::{request::RawRequest, IpcMessageBase},
    Status,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RPCType {
    Unary,
    ClientStreaming,
    ServerStreaming,
    Duplex,
}

pub enum Responses {
    Unary(IpcMessageBase),
    Streaming,
}

pub trait Responder<R: Runtime> {
    fn service_name(&self) -> String;
    fn get_rpc_type(&self, _route: &str) -> Option<RPCType> {
        None
    }
    fn has_rpc(&self, route: &str) -> bool {
        self.get_rpc_type(route).is_some()
    }
    fn respond(&self, message: RawRequest, webview: Webview<R>, resovler: InvokeResolver<R>);
}

pub struct Routes<R: Runtime>(HashMap<String, Arc<Box<dyn Responder<R> + Send + Sync + 'static>>>);

impl<R: Runtime> Default for Routes<R> {
    fn default() -> Self {
        Routes(HashMap::new())
    }
}

impl<R: Runtime> Routes<R> {
    pub fn insert_responder(&mut self, responder: impl Responder<R> + Send + Sync + 'static) {
        self.0.insert(responder.service_name(), Arc::new(Box::new(responder)));
    }
    pub fn add_responder(mut self, responder: impl Responder<R> + Send + Sync + 'static) -> Self {
        self.insert_responder(responder);
        self
    }
    pub fn respond(&self, message: RawRequest, webview: Webview<R>, resovler: InvokeResolver<R>) {
        let Some((_service_name, responder)) =
            self.0.iter().find(|(_, res)| res.has_rpc(&message.route))
        else {
            resovler.reject(Status::not_found(format!(
                "the {} rpc is not found",
                message.route
            )));
            return;
        };
        responder.respond(message, webview, resovler);
    }
}
