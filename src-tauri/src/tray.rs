use crate::app_state::AppState;
use crate::windowing::{hide_main_window, toggle_main_window};
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("toggle_panel", "Show Clipboard")
        .separator()
        .text("toggle_monitoring", "Pause Monitoring")
        .text("clear_history", "Clear History")
        .separator()
        .text("quit", "Quit")
        .build()?;

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("paste")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle_panel" => {
                let _ = toggle_main_window(app);
            }
            "toggle_monitoring" => {
                let state = app.state::<AppState>();
                state.toggle_monitoring();
                let _ = app.emit("clipboard-state-changed", true);
            }
            "clear_history" => {
                let app_handle = app.clone();
                let state = app.state::<AppState>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = state.store().clear_history().await;
                    let _ = app_handle.emit("clipboard-history-changed", true);
                    let _ = app_handle.emit("clipboard-state-changed", true);
                });
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = toggle_main_window(&tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }

    tray_builder.build(app)?;

    Ok(())
}

pub fn setup_window_lifecycle(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let managed_window = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = hide_main_window(&managed_window);
            }
        });
    }
}
