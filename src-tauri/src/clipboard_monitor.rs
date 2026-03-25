use crate::app_state::AppState;
use clipboard_rs::{Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext};
use tauri::{AppHandle, Emitter};

pub fn start_clipboard_monitor(app: AppHandle, state: AppState) {
    std::thread::spawn(move || {
        let clipboard = match ClipboardContext::new() {
            Ok(clipboard) => clipboard,
            Err(error) => {
                eprintln!("failed to initialize clipboard watcher context: {error}");
                return;
            }
        };

        let handler = ClipboardMonitor {
            app,
            state,
            clipboard,
        };

        let mut watcher = match ClipboardWatcherContext::new() {
            Ok(watcher) => watcher,
            Err(error) => {
                eprintln!("failed to initialize clipboard watcher: {error}");
                return;
            }
        };

        watcher.add_handler(handler);
        watcher.start_watch();
    });
}

struct ClipboardMonitor {
    app: AppHandle,
    state: AppState,
    clipboard: ClipboardContext,
}

impl ClipboardHandler for ClipboardMonitor {
    fn on_clipboard_change(&mut self) {
        if self.state.monitoring_paused() {
            return;
        }

        let text = self.clipboard.get_text().ok().filter(|value| !value.trim().is_empty());
        if let Some(text) = text {
            let app = self.app.clone();
            let state = self.state.clone();
            tauri::async_runtime::block_on(async move {
                if state.monitoring_paused() {
                    return;
                }

                if state.store().upsert_text(text).await.ok().flatten().is_some() {
                    let _ = app.emit("clipboard-history-changed", true);
                    let _ = app.emit("clipboard-state-changed", true);
                }
            });
            return;
        }

        if let Ok(image) = self.clipboard.get_image() {
            let app = self.app.clone();
            let state = self.state.clone();
            tauri::async_runtime::block_on(async move {
                if state.monitoring_paused() {
                    return;
                }

                if state.store().upsert_image(image).await.ok().flatten().is_some() {
                    let _ = app.emit("clipboard-history-changed", true);
                    let _ = app.emit("clipboard-state-changed", true);
                }
            });
        }
    }
}
