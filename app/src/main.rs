#![allow(warnings)]

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod core;
mod sniffer;
mod simulator;
mod handlers;
mod setup;
mod entity;
mod models;
mod misc;
mod database;
mod constants;
use anyhow::Result;
use log::LevelFilter;

use crate::constants::WINDOW_STATE_FLAGS;
use crate::handlers::generate_handlers;
use crate::misc::hook::setup_hook;

#[tokio::main]
async fn main() -> Result<()> {
    setup_hook();

    let mut log_builder = tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        .level_for("tao::platform_impl::platform::event_loop::runner", LevelFilter::Error)
        .max_file_size(5_000_000)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::LogDir {
                file_name: Some("loa_logs".to_string()),
            },
        ));

    #[cfg(debug_assertions)]
    {
        // log_builder = log_builder.target(tauri_plugin_log::Target::new(
        //     tauri_plugin_log::TargetKind::Stdout,
        // ));
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(log_builder.build())
        .setup(setup::setup_app)
        .on_window_event(misc::events::on_window_event)
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(WINDOW_STATE_FLAGS)
                .build(),
        )
        .invoke_handler(generate_handlers())
        .run(tauri::generate_context!())
        .expect("error while running application");

    Ok(())
}