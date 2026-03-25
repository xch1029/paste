use crate::windowing::toggle_main_window;
use anyhow::Result;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tauri::AppHandle;

pub fn register_global_hotkey(app: AppHandle) -> Result<()> {
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
    let hotkey_id = hotkey.id();
    let manager = Box::new(GlobalHotKeyManager::new()?);
    manager.register(hotkey)?;

    let app_handle = app.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.id == hotkey_id && event.state == HotKeyState::Pressed {
            let _ = toggle_main_window(&app_handle);
        }
    }));

    let _ = Box::leak(manager);
    Ok(())
}
