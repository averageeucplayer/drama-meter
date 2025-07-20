use tauri_plugin_window_state::StateFlags;

pub const DB_VERSION: i32 = 5;
pub const TIMEOUT_DELAY_MS: i64 = 1000;
pub const WORKSHOP_BUFF_ID: u32 = 9701;
pub const WINDOW_MS: i64 = 5_000;
pub const WINDOW_S: i64 = 5;
pub const PORT: u16 = 6040;
pub const LOCAL_PLAYERS_NAME: &'static str = "local_players.json";
pub const DB_NAME: &'static str = "encounters.db";
pub const SETTINGS_NAME: &'static str = "settings.json";
pub const REGION_NAME: &'static str = "current_region";
pub const GAME_STEAM_URI: [&'static str; 3] = ["/C", "start", "steam://rungameid/1599340"];
pub const METER_WINDOW_LABEL: &'static str = "main";
pub const METER_MINI_WINDOW_LABEL: &'static str = "mini";
pub const LOGS_WINDOW_LABEL: &'static str = "logs";
pub const WINDOW_STATE_FLAGS: StateFlags = StateFlags::from_bits_truncate(
    StateFlags::FULLSCREEN.bits()
        | StateFlags::MAXIMIZED.bits()
        | StateFlags::POSITION.bits()
        | StateFlags::SIZE.bits()
        | StateFlags::VISIBLE.bits(),
);
