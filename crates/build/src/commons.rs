use serde::Deserialize;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct KanaScope {
    pub pattern: String,
}

pub const COMMANDS: &[&str] = &["unary", "client_streaming", "server_streaming", "duplex"];
