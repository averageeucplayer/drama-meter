mod error;
mod encounter;
mod ui;
mod misc;
mod load;

use tauri::{generate_handler, ipc};

pub fn generate_handlers() -> Box<dyn Fn(ipc::Invoke) -> bool + Send + Sync> {
    Box::new(generate_handler![
        load::load,
        encounter::load_encounters_preview,
        encounter::load_encounter,
        encounter::get_encounter_count,
        encounter::open_most_recent_encounter,
        encounter::delete_encounter,
        encounter::delete_encounters,
        encounter::delete_encounters_below_min_duration,
        encounter::toggle_encounter_favorite,
        encounter::delete_all_encounters,
        encounter::delete_all_uncleared_encounters,
        encounter::get_sync_candidates,
        encounter::sync,
        ui::toggle_meter_window,
        ui::toggle_logs_window,
        ui::disable_blur,
        ui::enable_blur,
        ui::enable_aot,
        ui::disable_aot,
        ui::set_clickthrough,
        misc::open_url,
        misc::save_settings,
        misc::get_settings,
        misc::open_folder,
        misc::open_db_path,
        misc::get_db_info,
        misc::optimize_database,
        misc::check_start_on_boot,
        misc::set_start_on_boot,
        misc::check_loa_running,
        misc::start_loa_process,
        misc::remove_driver,
        misc::unload_driver,
    ])
}