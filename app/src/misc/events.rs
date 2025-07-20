use std::sync::Arc;

use anyhow::*;
use log::debug;
use tauri::{Manager, Window, WindowEvent};
use tauri_plugin_window_state::AppHandleExt;

use crate::{constants::{LOGS_WINDOW_LABEL, METER_MINI_WINDOW_LABEL, METER_WINDOW_LABEL, WINDOW_STATE_FLAGS}, misc::utils::CommandsManager};

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let label = window.label();
    
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();

            match label {
                LOGS_WINDOW_LABEL => {
                    window.hide().unwrap();
                }
                METER_MINI_WINDOW_LABEL => on_window_close(window).expect("An error occurred whilst closing app"),
                METER_WINDOW_LABEL => on_window_close(window).expect("An error occurred whilst closing app"),
                _ => {
                    debug!("Could not find handler for window: {}", label)
                }
            }
        },
        WindowEvent::Focused(_) => {
            window.app_handle().save_window_state(WINDOW_STATE_FLAGS).expect("failed to save window state");
        },
        _ => {},
    }
}

fn on_window_close(window: &Window) -> Result<()> {
    let app_handle = window.app_handle();
    let commands_manager = app_handle.state::<Arc<CommandsManager>>();
    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;
    let logs_window = app_handle.get_webview_window(LOGS_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;

    if logs_window.is_minimized()? {
        logs_window.unminimize()?
    }

    if meter_window.is_minimized()? {
        meter_window.unminimize()?;
    }

    app_handle
        .save_window_state(WINDOW_STATE_FLAGS)
        .expect("failed to save window state");
    
    commands_manager.unload_driver();

    app_handle.exit(0);

    Ok(())
}