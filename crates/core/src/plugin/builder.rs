use serde_json::Value as JsonValue;
use tauri::{webview::PageLoadPayload, AppHandle, RunEvent, Runtime, Url, Webview, Window};

use super::{
    KanamaruPlugin, OnDrop, OnEvent, OnNavigation, OnPageLoad, OnWebviewReady, OnWindowReady,
    SetupHook,
};
use crate::Routes;

/// Errors that can happen during [`Builder`].
#[derive(Debug, Clone, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum BuilderError {
    /// Plugin attempted to use a reserved name.
    #[error("plugin uses reserved name: {0}")]
    ReservedName(String),
}

const RESERVED_PLUGIN_NAMES: &[&str] = &["core", "tauri"];

pub struct Builder<R: Runtime> {
    routes: Routes<R>,
    name: &'static str,
    setup: Option<Box<SetupHook<R>>>,
    js_init_script: Option<String>,
    on_page_load: Box<OnPageLoad<R>>,
    on_webview_ready: Box<OnWebviewReady<R>>,
    on_event: Box<OnEvent<R>>,
    on_drop: Option<Box<OnDrop<R>>>,
    on_window_ready: Box<OnWindowReady<R>>,
    on_navigation: Box<OnNavigation<R>>,
}

impl<R> Builder<R>
where
    R: Runtime,
{
    /// Creates a new Plugin builder.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            setup: None,
            js_init_script: None,
            on_page_load: Box::new(|_, _| ()),
            on_webview_ready: Box::new(|_| ()),
            on_event: Box::new(|_, _| ()),
            on_drop: None,
            on_window_ready: Box::new(|_| ()),
            on_navigation: Box::new(|_, _| true),
            routes: Routes::default(),
        }
    }

    /// Same as [`tauri::plugin::Builder::js_init_script`]
    ///
    /// Sets the provided JavaScript to be run after the global object has been created,
    /// but before the HTML document has been parsed and before any other script included by the HTML document is run.
    ///
    /// Since it runs on all top-level document and child frame page navigations,
    /// it's recommended to check the `window.location` to guard your script from running on unexpected origins.
    ///
    /// The script is wrapped into its own context with `(function () { /* your script here */ })();`,
    /// so global variables must be assigned to `window` instead of implicitly declared.
    ///
    /// Note that calling this function multiple times overrides previous values.
    ///
    #[must_use]
    pub fn js_init_script(mut self, js_init_script: String) -> Self {
        self.js_init_script = Some(js_init_script);
        self
    }

    #[must_use]
    pub fn setup<F>(mut self, setup: F) -> Self
    where
        F: FnOnce(&AppHandle<R>, JsonValue, &Routes<R>) -> Result<(), Box<dyn std::error::Error>>
            + Send
            + 'static,
    {
        self.setup.replace(Box::new(setup));
        self
    }
    /// Callback invoked when the webview performs a navigation to a page.
    /// Same as [`tauri::plugin::Builder::on_page_load`]
    #[must_use]
    pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
    where
        F: FnMut(&Webview<R>, &PageLoadPayload<'_>) + Send + 'static,
    {
        self.on_page_load = Box::new(on_page_load);
        self
    }

    /// Same as [`tauri::plugin::Builder::on_webview_ready`]
    #[must_use]
    pub fn on_webview_ready<F>(mut self, on_webview_ready: F) -> Self
    where
        F: FnMut(Webview<R>) + Send + 'static,
    {
        self.on_webview_ready = Box::new(on_webview_ready);
        self
    }

    /// Same as [`tauri::plugin::Builder::on_event`]
    #[must_use]
    pub fn on_event<F>(mut self, on_event: F) -> Self
    where
        F: FnMut(&AppHandle<R>, &RunEvent) + Send + 'static,
    {
        self.on_event = Box::new(on_event);
        self
    }

    /// Same as [`tauri::plugin::Builder::on_drop`]
    #[must_use]
    pub fn on_drop<F>(mut self, on_drop: F) -> Self
    where
        F: FnOnce(AppHandle<R>) + Send + 'static,
    {
        self.on_drop.replace(Box::new(on_drop));
        self
    }

    /// Similar to [`tauri::plugin::Builder::on_navigation`]
    #[must_use]
    pub fn on_navigation<F>(mut self, on_navigation: F) -> Self
    where
        F: Fn(&Webview<R>, &Url) -> bool + Send + 'static,
    {
        self.on_navigation = Box::new(on_navigation);
        self
    }
    /// Similar to [`tauri::plugin::Builder::on_window_ready`]
    #[must_use]
    pub fn on_window_ready<F>(mut self, on_window_ready: F) -> Self
    where
        F: FnMut(Window<R>) + Send + 'static,
    {
        self.on_window_ready = Box::new(on_window_ready);
        self
    }
    /// Build the [`crate::KanamaruPlugin`]
    ///
    /// It returns an error if your plugin name is reserved.
    ///
    /// ## List of reserved names
    ///
    /// - core
    /// - tauri
    ///
    pub fn try_build(self) -> Result<KanamaruPlugin<R>, BuilderError> {
        if let Some(&reserved) = RESERVED_PLUGIN_NAMES.iter().find(|&r| r == &self.name) {
            return Err(BuilderError::ReservedName(reserved.into()));
        }
        Ok(KanamaruPlugin {
            name: self.name,
            app: None,
            setup: self.setup,
            js_init_script: self.js_init_script,
            on_page_load: self.on_page_load,
            on_webview_ready: self.on_webview_ready,
            on_event: self.on_event,
            on_drop: self.on_drop,
            on_navigation: self.on_navigation,
            on_window_ready: self.on_window_ready,
            routes: self.routes,
        })
    }
    /// Build the [`crate::KanamaruPlugin`]
    ///
    /// # Panics
    ///
    /// If the builder returns an error during [`Self::try_build`], then this method will panic.
    ///
    pub fn build(self) -> KanamaruPlugin<R> {
        self.try_build().unwrap()
    }
}
