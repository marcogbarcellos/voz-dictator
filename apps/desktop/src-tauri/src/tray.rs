use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

pub fn setup_tray(
    app: &tauri::App,
) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    let open_item = MenuItem::with_id(app, "open", "Open Dashboard", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Voz", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

    let tray = TrayIconBuilder::with_id("voz-tray")
        .icon(
            tauri::image::Image::from_bytes(include_bytes!("../icons/tray-idle.png"))
                .expect("failed to load tray icon"),
        )
        .icon_as_template(false)
        .tooltip("Voz — Voice Dictation")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => {
                if let Some(win) = app.get_webview_window("main") {
                    // Activate the app on macOS so it comes to the foreground
                    #[cfg(target_os = "macos")]
                    {
                        use cocoa::base::id;
                        use objc::{class, msg_send, sel, sel_impl};
                        unsafe {
                            let ns_app: id =
                                msg_send![class!(NSApplication), sharedApplication];
                            let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
                        }
                    }
                    let _ = win.show();
                    let _ = win.set_focus();
                    // Notify frontend to switch to dashboard view
                    let _ = app.emit("open-dashboard", ());
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(tray)
}
