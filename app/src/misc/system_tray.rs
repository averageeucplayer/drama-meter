use std::sync::Arc;

use anyhow::*;
use log::error;
use tauri::{async_runtime, menu::*, tray::{TrayIcon, TrayIconBuilder, TrayIconEvent}, App, LogicalPosition, LogicalSize, Manager, Position, Runtime, Size, Wry};
use tauri_plugin_window_state::{AppHandleExt, WindowExt};

use crate::{constants::*, misc::settings::SettingsManager, misc::utils::CommandsManager};


pub fn build(app: &App) -> Result<()> {
    let builder = TrayIconBuilder::new();

    let items: Vec<Box<dyn IsMenuItem<Wry>>> = vec![
        Box::new(MenuItemBuilder::new("Show Logs").id("show-logs").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Show Meter").id("show-meter").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Hide Meter").id("hide").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Start Lost Ark").id("start-loa").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Reset Window").id("reset").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Quit").id("quit").build(app)?) as Box<dyn IsMenuItem<Wry>>
    ];

    let item_refs: Vec<&dyn IsMenuItem<Wry>> = items.iter()
        .map(|b| b.as_ref())
        .collect();

    let menu = MenuBuilder::new(app)
        .items(&item_refs)
        .build()?;


    builder
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;

    Ok(())
}

pub fn on_menu_event<R: Runtime>(app: &tauri::AppHandle<R>, event: MenuEvent) {
    if let Err(err) = on_menu_event_inner(app, event) {
        error!("An error occurred whilst handling menu event {}", err);
    }
}

pub fn on_menu_event_inner<R: Runtime>(app: &tauri::AppHandle<R>, event: MenuEvent) -> Result<()> {
    let menu_item_id = event.id().0.as_str();
    let settings = app.state::<SettingsManager>();
    let commands = app.state::<Arc<CommandsManager>>();

    match menu_item_id {
        "quit" => {
            app.save_window_state(WINDOW_STATE_FLAGS)?;
            let commands = commands.clone();
            async_runtime::block_on(async {
                commands.unload_driver().await;
            });
            
            app.exit(0);
        }
        "hide" => {
            let meter_window = app.get_webview_window(METER_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;
            meter_window.hide()?;

            let logs_window = app.get_webview_window(METER_MINI_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;
            logs_window.hide()?;
        }
        "show-meter" => {
            let meter_window = app.get_webview_window(METER_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;
            meter_window.show()?;
            meter_window.unminimize()?;
            meter_window.set_ignore_cursor_events(false)?;
        }
        "load" => {
            let meter_window = app.get_webview_window(METER_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;
            meter_window.restore_state(WINDOW_STATE_FLAGS)?;
        }
        "save" => {
            app.save_window_state(WINDOW_STATE_FLAGS)?;
        }
        "reset" => {
            let meter_window = app.get_webview_window(METER_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;
            let size = Size::Logical(LogicalSize {
                width: 1280.0,
                height: 200.0,
            });
            meter_window.set_size(size)?;
            let position = Position::Logical(LogicalPosition { 
                x: 100.0,
                y: 100.0
            });
            meter_window.set_position(position)?;
            meter_window.show()?;
            meter_window.unminimize()?;
            meter_window.set_focus()?;
            meter_window.set_ignore_cursor_events(false)?;
        }
        "show-logs" => {
            let logs_window = app.get_webview_window(LOGS_WINDOW_LABEL).ok_or_else(|| anyhow!("Could not find window"))?;
            logs_window.show()?;
            logs_window.unminimize()?;
        }
        _ => {}
    }

    Ok(())
}

pub fn on_tray_icon_event<R: Runtime>(icon: &TrayIcon<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click { .. } => {
            let meter_window = icon.app_handle().get_webview_window(METER_WINDOW_LABEL).unwrap();
            meter_window.show().unwrap();
            meter_window.unminimize().unwrap();
            meter_window.set_ignore_cursor_events(false).unwrap();
        },
        _ => {}
    }
}