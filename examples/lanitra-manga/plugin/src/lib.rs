use kanamaru::KanamaruPlugin;
use tauri::Runtime;

pub fn init<R: Runtime>() -> KanamaruPlugin<R> {
    KanamaruPlugin::builder("lanitra-manga").build()
}
