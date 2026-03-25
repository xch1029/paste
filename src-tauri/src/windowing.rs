use tauri::{AppHandle, Manager, WebviewWindow};

pub fn toggle_main_window(app: &AppHandle) -> tauri::Result<()> {
    let window = app.get_webview_window("main").ok_or(tauri::Error::WindowNotFound)?;

    if window.is_visible()? {
        hide_main_window(&window)?;
    } else {
        show_main_window(&window)?;
    }

    Ok(())
}

pub fn show_main_window(window: &WebviewWindow) -> tauri::Result<()> {
    window.show()?;
    window.unminimize()?;
    window.set_focus()?;
    Ok(())
}

pub fn hide_main_window(window: &WebviewWindow) -> tauri::Result<()> {
    window.hide()?;
    Ok(())
}
