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
pub fn toggle_meter_window(app_handle: AppHandle, manager: State<'_, Mutex<SettingsManager>>) -> Result<(), AppError> {
    toggle_meter_window_inner(app_handle, manager).map_err(|_| AppError::UIConfig)?;

    Ok(())
}

fn toggle_meter_window_inner(app_handle: AppHandle, manager: State<'_, Mutex<SettingsManager>>) -> anyhow::Result<()> {
    let manager = manager.lock().unwrap();

    let settings = manager.get().unwrap();

    let label = if settings.general.mini {
        METER_MINI_WINDOW_LABEL
    } else {
        METER_WINDOW_LABEL
    };
    
    let meter = app_handle.get_webview_window(label).unwrap();

    if meter.is_visible()? {
        // workaround for tauri not handling minimized state for windows without decorations
        if meter.is_minimized()? {
            meter.unminimize()?;
        }
        meter.hide()?;
    } else {
        meter.show()?;
    }

    Ok(())
}

#[command]
pub fn toggle_logs_window(app_handle: AppHandle) -> Result<(), AppError> {
    toggle_logs_window_inner(app_handle).map_err(|_| AppError::UIConfig)?;

    Ok(())
}

fn toggle_logs_window_inner(app_handle: AppHandle) -> anyhow::Result<()> {
    let logs = app_handle.get_webview_window(LOGS_WINDOW_LABEL).unwrap();

     if logs.is_visible()? {
        logs.hide()?;
    } else {
        logs.emit("redirect-url", "logs")?;
        logs.show()?;
    }

    Ok(())
}

#[command]
pub fn disable_blur(app_handle: AppHandle) -> Result<(), AppError> {
    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).unwrap();
    clear_blur(&meter_window).map_err(|_| AppError::UIConfig)?;

    Ok(())
}

#[command]
pub fn enable_blur(app_handle: AppHandle) -> Result<(), AppError> {
    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).unwrap();
    let value = Some((10, 10, 10, 50));
    apply_blur(&meter_window, value).map_err(|_| AppError::UIConfig)?;

    Ok(())
}

#[command]
pub fn enable_aot(app_handle: AppHandle) -> Result<(), AppError> {
    
    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).unwrap();
    meter_window.set_always_on_top(true).map_err(|_| AppError::UIConfig)?;

    let mini_window = app_handle.get_webview_window(METER_MINI_WINDOW_LABEL).unwrap();
    mini_window.set_always_on_top(true).map_err(|_| AppError::UIConfig)?;

     Ok(())
}

#[command]
pub fn disable_aot(app_handle: AppHandle) -> Result<(), AppError> {
    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).unwrap();
    meter_window.set_always_on_top(false).map_err(|_| AppError::UIConfig)?;
    
    let mini_window = app_handle.get_webview_window(METER_MINI_WINDOW_LABEL).unwrap();
    mini_window.set_always_on_top(false).map_err(|_| AppError::UIConfig)?;

    Ok(())
}

#[command]
pub fn set_clickthrough(app_handle: AppHandle, set: bool) -> Result<(), AppError> {
    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).ok_or_else(|| AppError::UIConfig)?;

    meter_window.set_ignore_cursor_events(set).map_err(|_| AppError::UIConfig)?;

    Ok(())
}

