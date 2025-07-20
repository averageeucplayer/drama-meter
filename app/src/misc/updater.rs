use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use log::*;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::misc::utils::CommandsManager;

pub fn check(app: AppHandle, update_checked: Arc<AtomicBool>) {
    tauri::async_runtime::spawn(async move {
        info!("Checking available updates");
        let commands_manager = app.state::<Arc<CommandsManager>>();
        let updater = app.updater().unwrap();
        let result = updater.check().await.ok().flatten();

        match result {
            Some(manager) => {
                // #[cfg(not(debug_assertions))] {
                  
                // }

                info!(
                    "update available, downloading update: v{}",
                    manager.version
                );

                commands_manager.unload_driver();
                commands_manager.remove_driver().await;

                manager.download_and_install(|c, d| {}, || {}).await
                    .map_err(|e| {
                        error!("failed to download update: {}", e);
                    })
                    .ok();
            },
            None => {
                warn!("No updates available");
                update_checked.store(true, Ordering::Relaxed);
            },
        }
    });
}