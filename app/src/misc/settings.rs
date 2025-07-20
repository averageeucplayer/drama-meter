use anyhow::*;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub struct SettingsManager(PathBuf, Settings);

impl SettingsManager {
    pub fn new(path: PathBuf) -> Result<Mutex<Self>> {

        let settings = if path.exists() {
            let reader = File::open(&path)?;
            serde_json::from_reader(reader)?
        } else {
            let settings = Settings::default();
            let writer = File::create(&path)?;
            serde_json::to_writer_pretty(writer, &settings)?;
            settings
        };

        Ok(Mutex::new(Self(path, settings)))
    }

    pub fn get(&self) -> Result<&Settings> {
        Ok(&self.1)
    }

    pub fn save(&mut self, settings: Settings) -> Result<()> {
        let writer = File::create(&self.0)?;
        serde_json::to_writer_pretty(writer, &settings)?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub general: GeneralSettings,
    pub dev: DevSettings,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DevSettings {
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneralSettings {
    pub start_loa_on_start: bool,
    pub low_performance_mode: bool,
    #[serde(default = "default_true")]
    pub auto_iface: bool,
    pub port: u16,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub boss_only_damage: bool,
    #[serde(default = "default_true")]
    pub hide_meter_on_start: bool,
    pub hide_logs_on_start: bool,
    pub mini: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_true() -> bool {
    true
}