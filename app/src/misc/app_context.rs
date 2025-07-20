use std::path::PathBuf;

use crate::constants::*;

pub struct AppContext {
    pub version: String,
    pub app_name: String,
    pub resource_path: PathBuf,
    pub region_path: PathBuf,
    pub settings_path: PathBuf,
    pub database_path: PathBuf,
    pub local_players_path: PathBuf,
    pub current_exe: String
}

impl AppContext {
    pub fn new(app_name: String, resource_path: PathBuf, version: String) -> Self {
        Self {
            app_name: app_name,
            resource_path: resource_path.clone(),
            region_path: resource_path.clone().join(REGION_NAME),
            settings_path: resource_path.clone().join(SETTINGS_NAME),
            database_path: resource_path.clone().join(DB_NAME),
            local_players_path: resource_path.clone().join(LOCAL_PLAYERS_NAME),
            current_exe: std::env::current_exe().unwrap().to_string_lossy().to_string(),
            version,
        }
    }
}