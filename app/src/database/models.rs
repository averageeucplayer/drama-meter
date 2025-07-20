use std::collections::BTreeMap;

use chrono::{DateTime, TimeDelta, Utc};
use hashbrown::HashMap;

use crate::{entity::Entity, core::stats_api::PlayerStats, models::*};


pub struct EncounterDb {
    pub last_combat_packet: i64,
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub compressed_buffs: Vec<u8>,
    pub compressed_debuffs: Vec<u8>,
    pub total_shielding: u64,
    pub total_effective_shielding: u64,
    pub compressed_shields: Vec<u8>,
    pub misc: serde_json::Value,
    pub version: i32,
    pub compressed_boss_hp: Vec<u8>,
}

pub struct EncounterPreviewDb {
    pub encounter_id: i64,
    pub fight_start: i64,
    pub current_boss_name: String,
    pub duration: i64,
    pub preview_players: String,
    pub raid_difficulty: String,
    pub local_player: String,
    pub local_player_dps: i64,
    pub raid_clear: bool,
    pub boss_only_damage: bool,
}

pub struct EntityDb {
    pub name: String,
    pub encounter_id: i64,
    pub npc_id: u32,
    pub entity_type: String,
    pub class_id: u32,
    pub class: String,
    pub gear_score: f32,
    pub current_hp: i64,
    pub max_hp: i64,
    pub is_dead: bool,
    pub compressed_skills: Vec<u8>,
    pub compressed_damage_stats: Vec<u8>,
    pub skill_stats: serde_json::Value,
    pub dps: i64,
    pub character_id: u64,
    pub engraving_data: serde_json::Value,
    pub gear_hash:  Option<String>,
    pub ark_passive_active: Option<bool>,
    pub spec: Option<String>,
    pub ark_passive_data: serde_json::Value,
}

pub struct SaveToDb {
    pub duration: TimeDelta,
    pub boss_only_damage: bool,
    pub current_boss_name: String,
    pub local_player: String,
    pub started_on: DateTime<Utc>,
    pub updated_on: DateTime<Utc>,
    pub encounter_damage_stats: EncounterDamageStats,
    pub misc: EncounterMisc,
    pub entities: Vec<EncounterEntity>,
    pub damage_log: HashMap<u64, Vec<(i64, i64)>>,
    pub cast_log: HashMap<u64, HashMap<u32, Vec<i32>>>,
    pub boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    pub raid_clear: bool,
    pub party_info: Vec<Vec<String>>,
    pub raid_difficulty: String,
    pub region: Option<String>,
    pub player_info: Option<HashMap<String, PlayerStats>>,
    pub version: String,
    pub ntp_fight_start: i64,
    pub rdps_valid: bool,
    pub is_manual: bool,
    pub skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
}