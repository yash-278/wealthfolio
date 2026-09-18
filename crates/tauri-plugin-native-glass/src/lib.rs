#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_native_glass);

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("native-glass")
        .setup(|_app, _api| {
            #[cfg(target_os = "ios")]
            _api.register_ios_plugin(init_plugin_native_glass)?;
            Ok(())
        })
        .build()
}
