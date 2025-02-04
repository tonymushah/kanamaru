pub mod builder;
pub mod commands;
pub mod scope;

use commands::COMMANDS;
use regex::Regex;
use scope::KanaScope;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use tauri::{
    ipc::{CommandArg, CommandItem, CommandScope, GlobalScope, Invoke, InvokeBody},
    plugin::Plugin,
    webview::PageLoadPayload,
    AppHandle, Emitter, EventTarget, RunEvent, Runtime, Url, Webview, Window, WindowEvent,
};

use crate::{ipc::request::RawRequest, Routes, Status};

pub(crate) type SetupHook<R> = dyn FnOnce(&AppHandle<R>, JsonValue, &Routes<R>) -> Result<(), Box<dyn std::error::Error>>
    + Send;
pub(crate) type OnWebviewReady<R> = dyn FnMut(Webview<R>) + Send;
pub(crate) type OnEvent<R> = dyn FnMut(&AppHandle<R>, &RunEvent) + Send;
pub(crate) type OnPageLoad<R> = dyn FnMut(&Webview<R>, &PageLoadPayload<'_>) + Send;
pub(crate) type OnDrop<R> = dyn FnOnce(AppHandle<R>) + Send;
pub(crate) type OnWindowReady<R> = dyn FnMut(Window<R>) + Send;
pub(crate) type OnNavigation<R> = dyn Fn(&Webview<R>, &Url) -> bool + Send;

pub struct KanamaruPlugin<R: Runtime> {
    routes: Routes<R>,
    name: &'static str,
    app: Option<AppHandle<R>>,
    setup: Option<Box<SetupHook<R>>>,
    js_init_script: Option<String>,
    on_page_load: Box<OnPageLoad<R>>,
    on_webview_ready: Box<OnWebviewReady<R>>,
    on_event: Box<OnEvent<R>>,
    on_drop: Option<Box<OnDrop<R>>>,
    on_window_ready: Box<OnWindowReady<R>>,
    on_navigation: Box<OnNavigation<R>>,
}

impl<R> Plugin<R> for KanamaruPlugin<R>
where
    R: Runtime,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn initialize(
        &mut self,
        app: &AppHandle<R>,
        config: JsonValue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = config;
        self.app.replace(app.clone());
        if let Some(s) = self.setup.take() {
            (s)(app, config, &self.routes)?;
        }
        Ok(())
    }

    fn initialization_script(&self) -> Option<String> {
        self.js_init_script.clone()
    }

    fn on_page_load(&mut self, window: &Webview<R>, payload: &PageLoadPayload<'_>) {
        (self.on_page_load)(window, payload)
    }

    fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
        (self.on_event)(app, event)
    }
    fn window_created(&mut self, window: Window<R>) {
        (self.on_window_ready)(window)
    }

    fn webview_created(&mut self, webview: tauri::Webview<R>) {
        (self.on_webview_ready)(webview)
    }

    fn on_navigation(&mut self, webview: &tauri::Webview<R>, url: &tauri::Url) -> bool {
        (self.on_navigation)(webview, url)
    }
    fn extend_api(&mut self, invoke: tauri::ipc::Invoke<R>) -> bool {
        if !COMMANDS.contains(&invoke.message.command()) {
            invoke
                .resolver
                .reject(Status::invalid_argument("invalid command"));
            return true;
        }

        let payload = match invoke.message.payload() {
            InvokeBody::Json(v) => v,
            InvokeBody::Raw(_) => {
                invoke
                    .resolver
                    .reject(Status::invalid_argument("The invoke body is raw"));
                return true;
            }
        };

        let raw_reqwest = match RawRequest::deserialize(payload) {
            Ok(req) => req,
            Err(err) => {
                invoke.resolver.reject(Status::internal(err.to_string()));
                return true;
            }
        };

        let command_scope =
            match CommandScope::<KanaScope>::from_command(get_command_item("", self, &invoke)) {
                Ok(cs) => cs,
                Err(err) => {
                    invoke.resolver.reject(Status::internal(err.0.to_string()));
                    return true;
                }
            };

        let global_scope =
            match GlobalScope::<KanaScope>::from_command(get_command_item("", self, &invoke)) {
                Ok(cs) => cs,
                Err(err) => {
                    invoke.resolver.reject(Status::internal(err.0.to_string()));
                    return true;
                }
            };

        if !is_authorized(&raw_reqwest, &command_scope, &global_scope) {
            invoke.resolver.reject(Status::permission_denied(format!(
                "this route {} is not allowed to respond",
                raw_reqwest.route
            )));
            return true;
        }
        let webview = invoke.message.webview();
        {
            let cancel_token_event = raw_reqwest.cancel_token_event_id.clone();
            let webview_clone = webview.clone();
            webview.window().on_window_event(move |ev| {
                if let WindowEvent::Destroyed = ev {
                    let _ = webview_clone.emit_to(
                        EventTarget::webview(webview_clone.label()),
                        &cancel_token_event,
                        None::<()>,
                    );
                }
            });
        }
        self.routes
            .respond(raw_reqwest, invoke.message.webview(), invoke.resolver);
        true
    }
}

fn is_authorized(
    reqwest: &RawRequest,
    command_scope: &CommandScope<KanaScope>,
    global_scope: &GlobalScope<KanaScope>,
) -> bool {
    if !global_scope.allows().is_empty()
        && global_scope
            .allows()
            .iter()
            .flat_map(|sc| Regex::new(&sc.pattern))
            .any(|p| p.is_match(&reqwest.route))
    {
        return true;
    }
    if !global_scope.denies().is_empty()
        && global_scope
            .denies()
            .iter()
            .flat_map(|sc| Regex::new(&sc.pattern))
            .any(|p| p.is_match(&reqwest.route))
    {
        return false;
    }
    if !command_scope.allows().is_empty()
        && command_scope
            .allows()
            .iter()
            .flat_map(|sc| Regex::new(&sc.pattern))
            .any(|p| p.is_match(&reqwest.route))
    {
        return true;
    }
    if !command_scope.denies().is_empty()
        && command_scope
            .denies()
            .iter()
            .flat_map(|sc| Regex::new(&sc.pattern))
            .any(|p| p.is_match(&reqwest.route))
    {
        return false;
    }
    true
}

fn get_command_item<'a, R, P>(
    name: &'static str,
    plugin: &'a P,
    invoke: &'a Invoke<R>,
) -> CommandItem<'a, R>
where
    R: Runtime,
    P: Plugin<R>,
{
    CommandItem {
        plugin: Some(plugin.name()),
        name,
        key: name,
        message: &invoke.message,
        acl: &invoke.acl,
    }
}

impl<R: Runtime> Drop for KanamaruPlugin<R> {
    fn drop(&mut self) {
        if let (Some(on_drop), Some(app)) = (self.on_drop.take(), self.app.take()) {
            on_drop(app);
        }
    }
}
