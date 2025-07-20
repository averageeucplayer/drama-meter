use crate::handlers::error::AppError;
use crate::misc::app_context::AppContext;
use crate::constants::{LOGS_WINDOW_LABEL, METER_MINI_WINDOW_LABEL, METER_WINDOW_LABEL};
use crate::database::Database;
use crate::models::*;
use crate::misc::settings::{Settings, SettingsManager};
use crate::misc::utils::CommandsManager;
use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use tauri::{command, ipc, AppHandle, State};
use tauri::{Emitter, Manager};

#[command]
pub async fn open_db_path(manager: State<'_, CommandsManager>, app_context: State<'_, Arc<AppContext>>) -> Result<(), AppError> {

    manager
        .open_db_path(app_context.resource_path.to_str().unwrap())
        .await.map_err(|_| AppError::FileSystem)?;

    Ok(())
}

#[command]
pub async fn get_db_info(database: State<'_, Database>, min_duration: i64) -> Result<EncounterDbInfo, AppError> {
    
    let (count, filtered_count) = database.get_db_stats(min_duration).await.map_err(|_| AppError::Database)?;

    let size_str = database.get_metadata().map_err(|_| AppError::FileSystem)?;

    let result = EncounterDbInfo {
        size: size_str,
        total_encounters: count,
        total_encounters_filtered: filtered_count,
    };

    Ok(result)
}

#[command]
pub async fn optimize_database(database: State<'_, Database>) -> Result<(), AppError> {
    
    database.optimize().await.map_err(|_| AppError::Database)?;

    Ok(())
}

#[command]
pub async fn remove_driver(manager: State<'_, CommandsManager>) -> Result<(), AppError> {
    manager.remove_driver().await.map_err(|_| AppError::WindiverUnload)?;

    Ok(())
}

#[command]
pub async fn unload_driver(manager: State<'_, CommandsManager>) -> Result<(), AppError> {
    
    let is_success = manager.unload_driver().await.map_err(|_| AppError::WindiverUnload)?;

    if is_success {
        info!("stopped driver");
    }

    Ok(())
}

#[command]
pub async fn check_start_on_boot(manager: State<'_, CommandsManager>) -> Result<bool, AppError> {
    let is_set = manager.check_start_on_boot().await.map_err(|_| AppError::SetStartOnBoot)?;

    Ok(is_set)
}

#[command]
pub async fn set_start_on_boot(manager: State<'_, CommandsManager>, set: bool) -> Result<(), AppError> {
    
    manager.set_start_on_boot(set).await.map_err(|_| AppError::SetStartOnBoot)?;
    
    Ok(())
}

#[command]
pub async fn check_loa_running(manager: State<'_, CommandsManager>) -> Result<bool, AppError> {

    let is_running = manager.is_loa_running().await;

    Ok(is_running)
}

#[command]
pub async fn start_loa_process(manager: State<'_, CommandsManager>) -> Result<(), AppError> {

    let is_running = manager.is_loa_running().await;

    if !is_running {
        info!("lost ark already running");
        return Ok(())
    }

    manager.start_loa_process().await.map_err(|_| AppError::LoaProcessSpawn)?;
    info!("starting lost ark process...");

    Ok(())
}

#[command]
pub fn open_url(app_handle: AppHandle, url: String) -> Result<(), AppError> {
    let logs = app_handle.get_webview_window(LOGS_WINDOW_LABEL).unwrap();
    logs.emit("redirect-url", url).map_err(|_| AppError::FileSystem)?;

    Ok(())
}

#[command]
pub async fn save_settings(manager: State<'_, Mutex<SettingsManager>>, settings: Settings) -> Result<(), AppError> {
    manager.lock().unwrap().save(settings).map_err(|_| AppError::FileSystem)?;

    Ok(())
}

#[command]
pub async fn get_settings(manager: State<'_, Mutex<SettingsManager>>) -> Result<Settings, AppError> {
    let settings = manager.lock().unwrap().get().cloned().map_err(|_| AppError::Emit)?;
    
    Ok((settings))
}

#[command]
pub async fn open_folder(manager: State<'_, CommandsManager>, path: String) -> Result<(), AppError> {

    manager.open_folder(&path).await.map_err(|_| AppError::FileSystem)?;

    Ok(())
}