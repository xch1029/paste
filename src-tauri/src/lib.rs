mod app_state;
mod clipboard_monitor;
mod hotkey;
mod models;
mod storage;
mod tray;
mod windowing;

use app_state::AppState;
use models::{ActionResponse, AppStatus, HistoryResponse};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
async fn get_history(state: State<'_, AppState>, query: Option<String>) -> Result<HistoryResponse, String> {
    state
        .store()
        .list_items(query)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_app_state(state: State<'_, AppState>) -> Result<AppStatus, String> {
    state
        .store()
        .app_status(state.monitoring_paused())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn copy_item_to_clipboard(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<ActionResponse, String> {
    let response = state
        .store()
        .copy_item_to_clipboard(id)
        .await
        .map_err(|error| error.to_string())?;

    let _ = app.emit("clipboard-history-changed", true);
    let _ = app.emit("clipboard-state-changed", true);
    Ok(response)
}

#[tauri::command]
async fn toggle_item_pin(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<ActionResponse, String> {
    let response = state
        .store()
        .toggle_pin(id)
        .await
        .map_err(|error| error.to_string())?;

    let _ = app.emit("clipboard-history-changed", true);
    Ok(response)
}

#[tauri::command]
async fn delete_item(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<ActionResponse, String> {
    let response = state
        .store()
        .delete_item(id)
        .await
        .map_err(|error| error.to_string())?;

    let _ = app.emit("clipboard-history-changed", true);
    let _ = app.emit("clipboard-state-changed", true);
    Ok(response)
}

#[tauri::command]
async fn clear_history(app: AppHandle, state: State<'_, AppState>) -> Result<ActionResponse, String> {
    let response = state
        .store()
        .clear_history()
        .await
        .map_err(|error| error.to_string())?;

    let _ = app.emit("clipboard-history-changed", true);
    let _ = app.emit("clipboard-state-changed", true);
    Ok(response)
}

#[tauri::command]
async fn set_monitoring_paused(
    app: AppHandle,
    state: State<'_, AppState>,
    paused: bool,
) -> Result<AppStatus, String> {
    state.set_monitoring_paused(paused);

    let status = state
        .store()
        .app_status(state.monitoring_paused())
        .await
        .map_err(|error| error.to_string())?;

    let _ = app.emit("clipboard-state-changed", true);
    Ok(status)
}

#[tauri::command]
fn hide_panel(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window missing".to_string())?;
    windowing::hide_main_window(&window).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = tauri::async_runtime::block_on(AppState::initialize(app.handle()))
                .map_err(|error| tauri::Error::AssetNotFound(error.to_string()))?;

            app.manage(state.clone());

            hotkey::register_global_hotkey(app.handle().clone()).map_err(|error| {
                tauri::Error::AssetNotFound(format!("failed to register hotkey: {error}"))
            })?;

            tray::create_tray(app.handle())?;
            tray::setup_window_lifecycle(app.handle());
            clipboard_monitor::start_clipboard_monitor(app.handle().clone(), state);

            if let Some(window) = app.get_webview_window("main") {
                window.hide()?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            get_app_state,
            copy_item_to_clipboard,
            toggle_item_pin,
            delete_item,
            clear_history,
            set_monitoring_paused,
            hide_panel
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
