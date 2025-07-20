use log::*;
use tauri_plugin_opener::OpenerExt;
use tokio::runtime::Runtime;
use std::{sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
}, thread::{self, JoinHandle}};
use tauri::{App, AppHandle, Manager};
use tauri_plugin_window_state::WindowExt;

use crate::{constants::*, core::background_worker::{BackgroundWorker, BackgroundWorkerArgs}, database::Database, misc::{app_context::AppContext, settings::{Settings, SettingsManager}, system_tray, updater, utils::CommandsManager}, sniffer::PacketSniffer};

pub fn setup_app(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    system_tray::build(app)?;

    // #[cfg(debug_assertions)]
    // {
    //     meter_window.open_devtools();
    // }

    let app_handle = app.app_handle();
    let package_info = app.package_info();
    let version = package_info.version.to_string();
    let app_name = package_info.name.clone();
    let resource_path = app_handle
        .path()
        .resource_dir()?;

    let app_context = AppContext::new(app_name, resource_path.clone(), version.clone());
    let app_context = Arc::new(app_context);
    app.manage(app_context.clone());

    let commands_manager = Arc::new(CommandsManager::new(app_handle.clone(), app_context.clone()));
    app.manage(commands_manager.clone());

    let commands_manager_test = app.state::<Arc<CommandsManager>>();

    info!("test");

    let settings_manager = SettingsManager::new(app_context.settings_path.clone())?;
    let settings = settings_manager.lock().unwrap().get()?.clone();
    info!("settings loaded");
    app.manage(settings_manager);

    let database = Database::new(app_context.database_path.clone());
    let database = Arc::new(database);

    let migration_path = resource_path.join("assets/migration");
    match database.setup(migration_path) {
        Ok(_) => (),
        Err(e) => {
            warn!("error setting up database: {}", e);
        }
    }

    app.manage(database.clone());

    info!("starting app v{}", app_context.version);

    let update_checked = Arc::new(AtomicBool::new(false));
    updater::check(app_handle.clone(), update_checked.clone());

    setup_ui(&app_handle, &settings);

    let mut port = PORT;

    if settings.general.auto_iface && settings.general.port > 0 {
        port = settings.general.port;
    }

    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move { 

        let result = on_launch(
            version,
            update_checked,
            port,
            settings,
            app_context,
            app_handle,
            commands_manager,
            database).await;

        match result {
            Ok(_) => {

            },
            Err(err) => {
                error!("Fatal: {err:?}");
            },
        };
    });

    Ok(())
}

pub fn setup_ui(app_handle: &AppHandle, settings: &Settings) -> anyhow::Result<()> {
    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).unwrap();
    meter_window.restore_state(WINDOW_STATE_FLAGS)?;

    let mini_window = app_handle.get_webview_window(METER_MINI_WINDOW_LABEL).unwrap();
    meter_window.restore_state(WINDOW_STATE_FLAGS)?;

    let logs_window = app_handle.get_webview_window(LOGS_WINDOW_LABEL).unwrap();
    logs_window.restore_state(WINDOW_STATE_FLAGS)?;

    if settings.general.mini {
        mini_window.show()?;
    } else if !settings.general.hide_meter_on_start && !settings.general.mini {
        meter_window.show()?;
    }
    if !settings.general.hide_logs_on_start {
        logs_window.show()?;
    }
    if !settings.general.always_on_top {
        meter_window.set_always_on_top(false)?;
        mini_window.set_always_on_top(false)?;
    } else {
        meter_window.set_always_on_top(true)?;
        mini_window.set_always_on_top(true)?;
    }

     Ok(())
}

pub async fn on_launch(
    version: String,
    update_checked: Arc<AtomicBool>,
    port: u16,
    settings: Settings,
    app_context: Arc<AppContext>,
    app_handle: AppHandle,
    commands_manager: Arc<CommandsManager>,
    database: Arc<Database>
) -> anyhow::Result<()> {
    if settings.general.start_loa_on_start {
        info!("auto launch game enabled");

        if !commands_manager.is_loa_running().await {
            commands_manager.start_loa_process().await?;
        }
    }

    commands_manager.remove_driver().await;

    while !update_checked.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    info!("listening on port: {}", PORT);
    let packet_sniffer;

    #[cfg(feature = "meter-core")]
    {
        use crate::sniffer::WindivertSniffer;
        packet_sniffer = WindivertSniffer::new();
    }

    #[cfg(feature = "fake")]
    {
        packet_sniffer = FakeSniffer::new();
    }

    let packet_sniffer = Box::new(packet_sniffer) as Box<dyn PacketSniffer>;

    let args = BackgroundWorkerArgs {
        app: app_handle,
        context: app_context,
        database,
        packet_sniffer,
        port,
        settings,
        version
    };

    let mut background_worker = BackgroundWorker::new();
    background_worker.run();

    Ok(())
}