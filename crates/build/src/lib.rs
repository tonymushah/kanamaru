#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "prost")]
pub mod prost;
#[cfg(feature = "tauri")]
pub mod tauri;
pub mod utils;

#[cfg(feature = "tauri")]
pub use tauri::{build as plugin_build, get_tauri_plugin_builder};

#[cfg(feature = "prost")]
pub use prost::{
    builder::{compile_fds, compile_protos},
    ProstBuilder,
};
