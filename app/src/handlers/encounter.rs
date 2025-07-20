use crate::handlers::error::AppError;
use crate::misc::app_context::AppContext;
use crate::constants::{LOGS_WINDOW_LABEL, METER_MINI_WINDOW_LABEL, METER_WINDOW_LABEL};
use crate::database::Database;
use crate::models::*;
use crate::misc::settings::{Settings, SettingsManager};
use crate::misc::utils::CommandsManager;
use log::{error, info, warn};
use window_vibrancy::{apply_blur, clear_blur};
use std::sync::{Arc, Mutex};
use tauri::{command, ipc, AppHandle, State};
use tauri::{Emitter, Manager};

#[command]
pub async fn load_encounters_preview(
    database: State<'_, Database>,
    page: i32,
    page_size: i32,
    search: String,
    filter: SearchFilter,
) -> Result<EncountersOverview, AppError> {

    let (encounters, count) = database.load_encounters_preview(
        page,
        page_size,
        search,
        filter
    ).await.map_err(|_| AppError::Database)?;

    let result = EncountersOverview {
        encounters,
        total_encounters: count,
    };

    Ok(result)
}

#[command(async)]
pub async fn load_encounter(database: State<'_, Database>, id: String) -> Result<Encounter, AppError> {
  
    let encounter = database.load_encounter(id)
        .await.map_err(|_| AppError::Database)?;

    Ok(encounter)
}

#[command]
pub async fn get_sync_candidates(database: State<'_, Database>, force_resync: bool) -> Result<Vec<i32>, AppError> {

    let result = database
        .get_sync_candidates(force_resync)
        .await.map_err(|_| AppError::Database)?;

    Ok(result)
}

#[command]
pub async fn get_encounter_count(database: State<'_, Database>) -> Result<i32, AppError> {
    
    let count = database
        .get_encounter_count()
        .await.map_err(|_| AppError::Database)?;

    Ok(count)
}

#[command]
pub async fn open_most_recent_encounter(app_handle: AppHandle, database: State<'_, Database>) -> Result<(), AppError> {
   
    let id = database.get_last_encounter().await.map_err(|_| AppError::Database)?;

    
    let logs = app_handle.get_webview_window(LOGS_WINDOW_LABEL).unwrap();

    match id {
        Some(id) => {
            logs.emit("show-latest-encounter", id.to_string()).map_err(|_| AppError::Emit)?;
        }
        None => {
            logs.emit("redirect-url", "logs").map_err(|_| AppError::Emit)?;
        }
    }

    Ok(())
}

#[command]
pub async fn toggle_encounter_favorite(database: State<'_, Database>, id: i32) -> Result<(), AppError> {
    database.toggle_encounter_favorite(id).await.map_err(|_| AppError::Database)?;

    Ok(())
}

#[command]
pub async fn delete_encounter(database: State<'_, Database>, id: String) -> Result<(), AppError> {
   
    database.delete_encounter(id).await.map_err(|_| AppError::Database)?;

    Ok(())
}

#[command]
pub async fn delete_encounters(database: State<'_, Database>, ids: Vec<i32>) -> Result<(), AppError> {

    database.delete_encounters(ids).await.map_err(|_| AppError::Database)?;

    Ok(())
}

#[command]
pub async fn delete_encounters_below_min_duration(
    database: State<'_, Database>,
    min_duration: i64,
    keep_favorites: bool,
) -> Result<(), AppError> {
    
    database.delete_encounters_below_min_duration(min_duration, keep_favorites)
        .await.map_err(|_| AppError::Database)?;

    Ok(())
}

#[command]
pub async fn sync(database: State<'_, Database>, encounter: i32, upstream: String, failed: bool) -> Result<(), AppError> {

    database.insert_sync_log(encounter, upstream, failed)
        .await.map_err(|_| AppError::Database)?;

    Ok(())
}

#[command]
pub async fn delete_all_uncleared_encounters(database: State<'_, Database>, keep_favorites: bool) -> Result<(), AppError> {

    database.delete_all_uncleared_encounters(keep_favorites)
        .await.map_err(|_| AppError::Database)?;

    Ok(())
}

#[command]
pub async fn delete_all_encounters(database: State<'_, Database>, keep_favorites: bool) -> Result<(), AppError> {
    
    database.delete_all_encounters(keep_favorites)
        .await.map_err(|_| AppError::Database)?;

    Ok(())
}
