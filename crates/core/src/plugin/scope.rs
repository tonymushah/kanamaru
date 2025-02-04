use serde::Deserialize;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct KanaScope {
    pub pattern: String,
}
