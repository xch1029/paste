use crate::windowing::{toggle_main_window, toggle_picker_window};
use anyhow::Result;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tauri::AppHandle;

pub fn register_global_hotkey(app: AppHandle) -> Result<()> {
    let manage_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
    let picker_hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyV);
    let manage_hotkey_id = manage_hotkey.id();
    let picker_hotkey_id = picker_hotkey.id();
    let manager = Box::new(GlobalHotKeyManager::new()?);
    manager.register(manage_hotkey)?;
    manager.register(picker_hotkey)?;

    let app_handle = app.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state != HotKeyState::Pressed {
            return;
        }

        if event.id == manage_hotkey_id {
            let _ = toggle_main_window(&app_handle);
        } else if event.id == picker_hotkey_id {
            let _ = toggle_picker_window(&app_handle);
        }
    }));

    let _ = Box::leak(manager);
    Ok(())
}
