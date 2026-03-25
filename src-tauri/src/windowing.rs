use crate::app_state::AppState;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewWindow};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const PICKER_WINDOW_LABEL: &str = "picker";
const PICKER_MIN_WIDTH: u32 = 560;
const PICKER_HEIGHT: u32 = 198;
const PICKER_CARD_WIDTH: u32 = 248;
const PICKER_CARD_GAP: u32 = 16;
const PICKER_HORIZONTAL_PADDING: u32 = 36;
const PICKER_SCREEN_MARGIN: u32 = 40;
const PICKER_BOTTOM_MARGIN: i32 = 24;
const PICKER_VISIBLE_LIMIT: usize = 24;

pub fn toggle_main_window(app: &AppHandle) -> tauri::Result<()> {
    toggle_window(app, MAIN_WINDOW_LABEL)
}

pub fn toggle_picker_window(app: &AppHandle) -> tauri::Result<()> {
    toggle_window(app, PICKER_WINDOW_LABEL)
}

pub fn hide_window_by_label(app: &AppHandle, label: &str) -> tauri::Result<()> {
    let window = get_window(app, label)?;
    hide_window(&window)
}

pub fn sync_picker_window(app: &AppHandle) -> tauri::Result<()> {
    let window = get_window(app, PICKER_WINDOW_LABEL)?;
    sync_picker_window_layout(app, &window)?;
    position_picker_window(app, &window)?;
    Ok(())
}

fn toggle_window(app: &AppHandle, label: &str) -> tauri::Result<()> {
    let window = get_window(app, label)?;

    if window.is_visible()? {
        hide_window(&window)?;
    } else if label == PICKER_WINDOW_LABEL {
        sync_picker_window(app)?;
        show_window(&window)?;
    } else {
        show_window(&window)?;
    }

    Ok(())
}

fn get_window(app: &AppHandle, label: &str) -> tauri::Result<WebviewWindow> {
    app.get_webview_window(label).ok_or(tauri::Error::WindowNotFound)
}

fn show_window(window: &WebviewWindow) -> tauri::Result<()> {
    window.show()?;
    window.unminimize()?;
    window.set_focus()?;
    Ok(())
}

fn hide_window(window: &WebviewWindow) -> tauri::Result<()> {
    window.hide()?;
    Ok(())
}

fn sync_picker_window_layout(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    let visible_count = app
        .try_state::<AppState>()
        .map(|state| {
            tauri::async_runtime::block_on(state.store().list_items(None))
                .map(|response| response.items.len().min(PICKER_VISIBLE_LIMIT))
        })
        .transpose()
        .map_err(|error| tauri::Error::AssetNotFound(error.to_string()))?
        .unwrap_or(1)
        .max(1);

    let monitor = window
        .current_monitor()?
        .or(app.primary_monitor()?)
        .ok_or(tauri::Error::WindowNotFound)?;
    let work_area = monitor.work_area();
    let desired_width = PICKER_HORIZONTAL_PADDING
        + visible_count as u32 * PICKER_CARD_WIDTH
        + visible_count.saturating_sub(1) as u32 * PICKER_CARD_GAP;
    let max_width = work_area.size.width.saturating_sub(PICKER_SCREEN_MARGIN);
    let width = desired_width.clamp(PICKER_MIN_WIDTH, max_width.max(PICKER_MIN_WIDTH));

    window.set_size(Size::Physical(PhysicalSize::new(width, PICKER_HEIGHT)))?;
    Ok(())
}

fn position_picker_window(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    let monitor = window.current_monitor()?.or(app.primary_monitor()?).ok_or(tauri::Error::WindowNotFound)?;
    let work_area = monitor.work_area();
    let window_size = window.outer_size()?;
    let x = work_area.position.x + ((work_area.size.width as i32 - window_size.width as i32) / 2).max(0);
    let y = work_area.position.y + work_area.size.height as i32 - window_size.height as i32 - PICKER_BOTTOM_MARGIN;

    window.set_position(Position::Physical(PhysicalPosition::new(x, y.max(work_area.position.y))))?;
    Ok(())
}
