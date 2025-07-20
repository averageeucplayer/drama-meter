use std::io::Write;
use std::sync::Arc;

use log::*;
use serde::Serialize;
use sysinfo::System;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use flate2::write::GzEncoder;
use flate2::Compression;
use anyhow::Result;

use crate::misc::app_context::AppContext;
use crate::constants::*;

pub struct CommandsManager(AppHandle, Arc<AppContext>);

impl CommandsManager {
    pub fn new(handle: AppHandle, context: Arc<AppContext>) -> Self {
        Self(handle, context)
    }

    pub async fn remove_driver(&self) -> Result<()> {
        self.0.shell().command("sc")
            .args(["delete", "windivert"]).output().await?;
        Ok(())
    }

    pub async fn unload_driver(&self) -> Result<bool> {
        let success = self.0.shell().command("sc")
            .args(["sc", "stop", "windivert"])
            .output().await.ok()
            .map(|pr| pr.status.success())
            .unwrap_or_default();

        Ok(success)
    }

    pub async fn open_db_path(&self, path: &str) -> Result<()> {
        self.0.shell().command("explorer")
            .args([path])
            .spawn();

        Ok(())
    }

    pub async fn check_start_on_boot(&self) -> Result<bool> {

        let output = self.0.shell().command("schtasks")
            .args(["/query", "/tn", "LOA_Logs_Auto_Start"])
            .output()
            .await?;

        Ok(output.status.success())
    }

    pub async fn open_folder(&self, path: &str) -> Result<()> {

        let mut path = path.to_string();
        if path.contains("USERPROFILE") {
            if let Ok(user_dir) = std::env::var("USERPROFILE") {
                path = path.replace("USERPROFILE", user_dir.as_str());
            }
        }
        
        info!("open_folder: {}", path);

        self.0.shell().command("explorer").args([path.as_str()]).spawn().ok();

        Ok(())
    }

    pub async fn is_loa_running(&self) -> bool {
        let system = System::new_all();
        let process_name = "lostark.exe";

        for process in system.processes().values() {
            if process.name().to_string_lossy().to_ascii_lowercase() == process_name {
                return true;
            }
        }

        false
    }

    pub async fn start_loa_process(&self) -> Result<()> {
        self.0.shell()
            .command("cmd")
            .args(GAME_STEAM_URI)
            .spawn()?;

        Ok(())
    }

    pub async fn set_start_on_boot(&self, set: bool) -> Result<()> {

        let task_name = "LOA_Logs_Auto_Start";
        let current_exe = std::env::current_exe()?;
        let app_path = current_exe.to_string_lossy().to_string();
        let args = ["/delete", "/tn", task_name, "/f"];

        if set {
            self.0.shell()
                .command("schtasks").args(args)
                .output().await.ok();

            let args = [
                "/create",
                "/tn",
                task_name,
                "/tr",
                &format!("\"{}\"", &app_path),
                "/sc",
                "onlogon",
                "/rl",
                "highest",
            ];

            let output = self.0.shell().command("schtasks")
                .args(args).output().await;
        } else {
            let args = ["/delete", "/tn", task_name, "/f"];

            let output = self.0.shell().command("schtasks")
                .args(args).output().await;
            }

        Ok(())
    }
}

pub fn compress_json<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let bytes = serde_json::to_vec(value)?;
    encoder.write_all(&bytes)?;
    encoder.finish()?;

    Ok(bytes)
}

// #[command]
// async fn open_db_path(app_handle: AppHandle) -> Result<(), AppError> {
//     let path = window
//         .app_handle()
//         .path()
//         .resource_dir()
//         .expect("could not get resource dir");
//     info!("open_db_path: {}", path.display());
    
// }
