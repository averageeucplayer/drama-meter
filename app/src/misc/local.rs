use std::{fs::File, path::PathBuf};
use anyhow::*;
use chrono::{DateTime, Utc};
use hashbrown::HashMap;
use uuid::Uuid;
use crate::models::{LocalInfo, LocalPlayer};

pub struct LocalManager(PathBuf, LocalInfo);

impl LocalManager {
    pub fn new(path: PathBuf) -> Result<Self> {

        let local_info = if path.exists() {
            let reader = File::open(&path)?;
            serde_json::from_reader(reader)?
        }
        else {
            let mut local_info = LocalInfo::default();
            local_info.client_id = Uuid::new_v4().to_string();

            let writer = File::create(&path)?;
            serde_json::to_writer_pretty(writer, &local_info)?;
            local_info
        };

        Ok(Self(path, local_info))
    }

    pub fn get(&self) -> &HashMap<u64, LocalPlayer> {
        &self.1.local_players
    }

    pub fn write(&mut self, name: String, character_id: u64, recorded_on: DateTime<Utc>) -> Result<()> {

        self.1.updated_on = recorded_on;
        self.1.local_players
            .entry(character_id)
            .and_modify(|e| {
                e.name = name.clone();
                e.count += 1;
            })
            .or_insert(LocalPlayer {
                name: name.clone(),
                count: 1,
            });

        let writer = File::create(&self.0)?;
        serde_json::to_writer_pretty(writer, &self.1)?;

        Ok(())
    }
}

           

    // read saved local players
    // this info is used in case meter was opened late
    // let mut local_info: LocalInfo = LocalInfo::default();
    // let mut local_player_path = app.path().resource_dir().unwrap();
    // let mut client_id = "".to_string();
    // local_player_path.push("local_players.json");

    // if local_player_path.exists() {
    //     let local_players_file = std::fs::read_to_string(local_player_path.clone())?;
    //     local_info = serde_json::from_str(&local_players_file).unwrap_or_default();
    //     client_id = local_info.client_id.clone();
    //     let valid_id = Uuid::try_parse(client_id.as_str()).is_ok();
    //     if client_id.is_empty() || !valid_id {
    //         client_id = Uuid::new_v4().to_string();
    //         stats_api.client_id.clone_from(&client_id);
    //         local_info.client_id.clone_from(&client_id);
    //         write_local_players(&local_info, &local_player_path)?;
    //     } else {
    //         stats_api.client_id.clone_from(&client_id);
    //     }
    // }