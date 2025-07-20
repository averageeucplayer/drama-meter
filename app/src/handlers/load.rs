use crate::handlers::error::AppError;
use crate::misc::app_context::{self, AppContext};
use crate::database::Database;
use crate::misc::data::*;
use crate::models::*;
use crate::misc::settings::{Settings, SettingsManager};
use crate::misc::utils::CommandsManager;
use chrono::Utc;
use hashbrown::HashMap;
use log::{error, info, warn};
use strum::VariantNames;
use std::sync::{Arc, Mutex};
use tauri::{command, ipc, AppHandle, State};
use tauri::{Emitter, Manager};

#[command]
pub async fn load(app_context: State<'_, Arc<AppContext>>) -> Result<LoadResult, AppError> {

    let esther_name_to_icon = ESTHER_DATA.iter().map(|pr| (pr.name.as_str(), pr.icon.as_str())).collect();
    let result = LoadResult {
        loaded_on: Utc::now(),
        version: app_context.version.clone(),
        app_name: app_context.app_name.clone(),
        esther_name_to_icon,
        arkPassiveIdToSpec: &ARK_PASSIVE_ID_TO_SPEC,
        arkPassives: &ARK_PASSIVE_DATA,
        boss_hp_map: &BOSS_HP_MAP,
        encounterMap: &ENCOUNTER_MAP,
        difficultyMap: RaidDifficulty::VARIANTS.to_vec(),
        raid_gates: &RAID_MAP,
        guardianRaidBosses: &GUARDIAN_RAID_BOSSES,
        classesMap: &CLASS_MAP,
        classNameToClassId: &REVERSE_CLASS_MAP,
        classes: &CLASS_NAMES,
        cardMap: &CARD_MAP,
        esthers: &ESTHER_DATA,
        card_ids: CARD_MAP.keys().into_iter().cloned().collect(),
        support_class_ids: [Class::Bard as u32, Class::Paladin as u32, Class::Artist as u32].to_vec()
    };

    Ok(result)
}
