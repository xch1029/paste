use crate::app_state::AppState;
use crate::windowing::{hide_window_by_label, toggle_main_window, toggle_picker_window, MAIN_WINDOW_LABEL, PICKER_WINDOW_LABEL};
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("show_management", "打开管理页")
        .text("show_picker", "打开快速列表")
        .separator()
        .text("toggle_monitoring", "暂停/恢复监听")
        .text("clear_history", "清空历史")
        .separator()
        .text("quit", "退出")
        .build()?;

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("paste")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_management" => {
                let _ = toggle_main_window(app);
            }
            "show_picker" => {
                let _ = toggle_picker_window(app);
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
                let _ = toggle_picker_window(&tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }

    tray_builder.build(app)?;

    Ok(())
}

pub fn setup_window_lifecycle(app: &AppHandle) {
    for label in [MAIN_WINDOW_LABEL, PICKER_WINDOW_LABEL] {
        if let Some(window) = app.get_webview_window(label) {
            let app_handle = app.clone();
            let window_label = label.to_string();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = hide_window_by_label(&app_handle, &window_label);
                }
            });
        }
    }
}
