use serde::Deserialize;
use tauri_plugin::Builder;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct KanaScope {
    pub pattern: String,
}

pub const COMMANDS: &[&str] = &["unary", "client_streaming", "server_streaming", "duplex"];

pub fn get_tauri_plugin_builder() -> Builder<'static> {
    Builder::new(COMMANDS).global_scope_schema(schemars::schema_for!(KanaScope))
}

pub fn build() {
    get_tauri_plugin_builder().build();
}
