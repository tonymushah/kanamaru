pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod rpc;

use client::setup_client;
use config::Config;
use kanamaru::KanamaruPlugin;
use serde::Deserialize;
use tauri::Runtime;
const PLUGIN_NAME: &str = "lanitra-manga";
pub fn init<R: Runtime>() -> KanamaruPlugin<R> {
    KanamaruPlugin::builder(PLUGIN_NAME)
        .setup(|app, config, _routes| {
            let config = Config::deserialize(&config)?;
            tauri::async_runtime::block_on(setup_client(app, &config))?;
            Ok(())
        })
        .build()
}
