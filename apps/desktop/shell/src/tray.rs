use anyhow::Result;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition,
};

// Bundled at compile time so the binary doesn't depend on a runtime path.
const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/tray.png");

pub fn build(app: &AppHandle) -> Result<()> {
    let dashboard = MenuItem::with_id(app, "dashboard", "Open Dashboard…", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Pulse", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&dashboard, &settings, &separator, &quit])?;

    let icon = Image::from_bytes(TRAY_ICON_PNG)?;

    let _tray = TrayIconBuilder::with_id("pulse-tray")
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "dashboard" => show(app, "dashboard"),
            "settings" => show(app, "settings"),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("popover") {
                    if win.is_visible().unwrap_or(false) {
                        let _ = win.hide();
                    } else {
                        // Anchor under the click; offset half the popover width.
                        let x = (position.x - 160.0) as i32;
                        let y = position.y as i32;
                        let _ = win.set_position(PhysicalPosition::new(x.max(0), y.max(0)));
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn show(app: &AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.show();
        let _ = w.set_focus();
    }
}
