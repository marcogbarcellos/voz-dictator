use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn setup_hotkey(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();

    app.global_shortcut().on_shortcut("Alt+Space", move |_app, shortcut, event| {
        if event.state == ShortcutState::Pressed {
            log::info!("Hotkey pressed: {}", shortcut);
            let _ = app_handle.emit("hotkey-toggle", ());
        }
    })?;

    log::info!("Global hotkey registered: Alt+Space");
    Ok(())
}
