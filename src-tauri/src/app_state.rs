use crate::storage::AppStore;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct AppState {
    store: AppStore,
    monitoring_paused: Arc<AtomicBool>,
}

impl AppState {
    pub async fn initialize(app: &AppHandle) -> Result<Self> {
        let data_dir = app.path().app_data_dir()?;
        let store = AppStore::new(&data_dir).await?;

        Ok(Self {
            store,
            monitoring_paused: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn store(&self) -> &AppStore {
        &self.store
    }

    pub fn monitoring_paused(&self) -> bool {
        self.monitoring_paused.load(Ordering::SeqCst)
    }

    pub fn set_monitoring_paused(&self, paused: bool) {
        self.monitoring_paused.store(paused, Ordering::SeqCst);
    }

    pub fn toggle_monitoring(&self) {
        let next = !self.monitoring_paused();
        self.set_monitoring_paused(next);
    }
}
