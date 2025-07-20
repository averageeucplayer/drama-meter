use strum::VariantNames;
use strum_macros::{VariantArray, VariantNames};
use strum_macros::{AsRefStr, EnumString};
use std::fmt::Display;
use std::mem::transmute;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use bitflags::bitflags;
use chrono::Duration;
use hashbrown::{HashMap, HashSet};
use meter_core::packets::definitions::PKTSkillDamageNotify;
use meter_core::packets::structures::SkillDamageEvent;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use serde_with::serde_as;
use serde_with::DefaultOnError;

use crate::entity::Entity;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadResult<'a> {
    pub loaded_on: DateTime<Utc>,
    pub version: String,
    pub app_name: String,
    pub esther_name_to_icon: HashMap<&'a str, &'a str>,
    pub esthers: &'a Vec<Esther>,
    pub arkPassiveIdToSpec: &'a HashMap<u32, String>,
    pub arkPassives: &'a HashMap<u32, ArkPassiveInfo>,
    pub boss_hp_map: &'a HashMap<String, u32>,
    pub encounterMap: &'a HashMap<String, HashMap<String, Vec<String>>>,
    pub difficultyMap: Vec<&'a str>,
    pub raid_gates: &'a HashMap<String, String>,
    pub guardianRaidBosses: &'a Vec<&'a str>,
    pub classesMap: &'a HashMap<u32, &'a str>,
    pub classNameToClassId: &'a HashMap<&'a str, u32>,
    pub classes: &'a Vec<&'a str>,
    pub cardMap: &'a HashMap<u32, Card>,
    pub card_ids: Vec<u32>,
    pub support_class_ids: Vec<u32>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkPassiveInfo {
    #[serde(rename = "0")]
    pub name: String,
    #[serde(rename = "1")]
    pub icon: String,
    #[serde(rename = "2")]
    pub value1: i32,
    #[serde(rename = "3")]
    pub value2: i32,
    #[serde(rename = "4")]
    pub value3: i32,
    #[serde(rename = "5")]
    pub value4: i32,
    #[serde(rename = "6")]
    pub value5: i32
}

#[derive(Default, Debug, Copy, Clone, AsRefStr, PartialEq, EnumString, VariantArray)]
#[repr(u32)]
pub enum Class {
    #[default]
    Unknown = 0,
    #[strum(serialize = "Warrior (Male)")]
    WarriorMale = 101,
    Berserker = 102,
    Destroyer = 103,
    Gunlancer = 104,
    Paladin = 105,
    #[strum(serialize = "Warrior (Female)")]
    WarriorFemale = 111,
    Slayer = 112,
    Mage = 201,
    Arcanist = 202,
    Summoner = 203,
    Bard = 204,
    Sorceress = 205,
    #[strum(serialize = "Martial Artist (Female)")]
    MartialArtistFemale = 301,
    Wardancer = 302,
    Scrapper = 303,
    Soulfist = 304,
    Glaivier = 305,
    #[strum(serialize = "Martial Artist (Male)")]
    MartialArtistMale = 311,
    Striker = 312,
    Breaker = 313,
    Assassin = 401,
    Deathblade = 402,
    Shadowhunter = 403,
    Reaper = 404,
    Souleater = 405,
    #[strum(serialize = "Gunner (Male)")]
    GunnerMale = 501,
    Sharpshooter = 502,
    Deadeye = 503,
    Artillerist = 504,
    Machinist = 505,
    #[strum(serialize = "Gunner (Female)")]
    GunnerFemale = 511,
    Gunslinger = 512,
    Specialist = 601,
    Artist = 602,
    Aeromancer = 603,
    Wildsoul = 604,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum EntityType {
    #[default]
    Unknown = 0,
    Monster = 1,
    Boss = 2,
    Guardian = 3,
    Player = 4,
    Npc = 5,
    Esther = 6,
    Projectile = 7,
    Summon = 8,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EntityType::Unknown => "UNKNOWN",
            EntityType::Monster => "MONSTER",
            EntityType::Boss => "BOSS",
            EntityType::Guardian => "GUARDIAN",
            EntityType::Player => "PLAYER",
            EntityType::Npc => "NPC",
            EntityType::Esther => "ESTHER",
            EntityType::Projectile => "PROJECTILE",
            EntityType::Summon => "SUMMON",
        };
        write!(f, "{s}")
    }
}

impl FromStr for EntityType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "UNKNOWN" => Ok(EntityType::Unknown),
            "MONSTER" => Ok(EntityType::Monster),
            "BOSS" => Ok(EntityType::Boss),
            "GUARDIAN" => Ok(EntityType::Guardian),
            "PLAYER" => Ok(EntityType::Player),
            "NPC" => Ok(EntityType::Npc),
            "ESTHER" => Ok(EntityType::Esther),
            "PROJECTILE" => Ok(EntityType::Projectile),
            "SUMMON" => Ok(EntityType::Summon),
            _ => Ok(EntityType::Unknown),
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub last_combat_packet: i64,
    pub fight_start: i64,
    pub local_player: String,
    pub entities: HashMap<String, EncounterEntity>,
    pub current_boss_name: String,
    pub current_boss: Option<EncounterEntity>,
    pub encounter_damage_stats: EncounterDamageStats,
    pub duration: i64,
    pub difficulty: Option<String>,
    pub favorite: bool,
    pub cleared: bool,
    pub boss_only_damage: bool,
    pub sync: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDamageStats {
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub buffs: HashMap<u32, StatusEffect>,
    pub debuffs: HashMap<u32, StatusEffect>,
    pub total_shielding: u64,
    pub total_effective_shielding: u64,
    pub applied_shield_buffs: HashMap<u32, StatusEffect>,
    #[serde(skip)]
    pub unknown_buffs: HashSet<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misc: Option<EncounterMisc>,
    pub boss_hp_log: HashMap<String, Vec<BossHpLog>>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterEntity {
    pub id: u64,
    pub character_id: u64,
    pub npc_id: u32,
    pub name: String,
    pub entity_type: EntityType,
    pub class_id: u32,
    pub class: String,
    pub gear_score: f32,
    pub current_hp: i64,
    pub max_hp: i64,
    pub current_shield: u64,
    pub is_dead: bool,
    pub skills: HashMap<u32, Skill>,
    pub damage_stats: DamageStats,
    pub skill_stats: SkillStats,
    pub engraving_data: Option<Vec<String>>,
    pub gear_hash: Option<String>,
    pub ark_passive_active: Option<bool>,
    pub ark_passive_data: Option<ArkPassiveData>,
    pub spec: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub total_damage: i64,
    pub max_damage: i64,
    pub max_damage_cast: i64,
    pub buffed_by: HashMap<u32, i64>,
    pub debuffed_by: HashMap<u32, i64>,
    pub buffed_by_support: i64,
    pub buffed_by_identity: i64,
    pub buffed_by_hat: i64,
    pub debuffed_by_support: i64,
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub adjusted_crit: Option<f64>,
    pub crit_damage: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub back_attack_damage: i64,
    pub front_attack_damage: i64,
    pub dps: i64,
    pub cast_log: Vec<i32>,
    pub tripod_index: Option<TripodIndex>,
    pub tripod_level: Option<TripodLevel>,
    pub gem_cooldown: Option<u8>,
    pub gem_tier: Option<u8>,
    pub gem_damage: Option<u8>,
    pub gem_tier_dmg: Option<u8>,
    #[serde(skip)]
    pub tripod_data: Option<Vec<TripodData>>,
    #[serde(skip)]
    pub summon_sources: Option<Vec<u32>>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
    pub rdps_damage_given: i64,
    pub skill_cast_log: Vec<SkillCast>,
    #[serde(skip)]
    pub updated_on: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillSlim {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub name: String,
    pub icon: String,
    pub is_hyper_awakening: bool,
    pub sources: Vec<u32>
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct TripodData {
    pub index: u8,
    pub options: Vec<SkillFeatureOption>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase", default)]
pub struct TripodLevel {
    pub first: u16,
    pub second: u16,
    pub third: u16,
}

impl PartialEq for TripodLevel {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.second == other.second && self.third == other.third
    }
}

impl Eq for TripodLevel {}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase", default)]
pub struct TripodIndex {
    pub first: u8,
    pub second: u8,
    pub third: u8,
}

impl PartialEq for TripodIndex {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.second == other.second && self.third == other.third
    }
}

impl Eq for TripodIndex {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ArkPassiveData {
    pub evolution: Option<Vec<ArkPassiveNode>>,
    pub enlightenment: Option<Vec<ArkPassiveNode>>,
    pub leap: Option<Vec<ArkPassiveNode>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ArkPassiveNode {
    pub id: u32,
    pub lv: u8,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct DamageStats {
    pub damage_dealt: i64,
    pub hyper_awakening_damage: i64,
    pub damage_taken: i64,
    pub buffed_by: HashMap<u32, i64>,
    pub debuffed_by: HashMap<u32, i64>,
    pub buffed_by_support: i64,
    pub buffed_by_identity: i64,
    pub debuffed_by_support: i64,
    pub buffed_by_hat: i64,
    pub crit_damage: i64,
    pub back_attack_damage: i64,
    pub front_attack_damage: i64,
    pub shields_given: u64,
    pub shields_received: u64,
    pub damage_absorbed: u64,
    pub damage_absorbed_on_others: u64,
    pub shields_given_by: HashMap<u32, u64>,
    pub shields_received_by: HashMap<u32, u64>,
    pub damage_absorbed_by: HashMap<u32, u64>,
    pub damage_absorbed_on_others_by: HashMap<u32, u64>,
    pub deaths: i64,
    pub death_time: i64,
    pub dps: i64,
    #[serde(default)]
    pub dps_average: Vec<i64>,
    #[serde(default)]
    pub dps_rolling_10s_avg: Vec<i64>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
    pub rdps_damage_given: i64,
    #[serde(default)]
    pub incapacitations: Vec<IncapacitatedEvent>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub counters: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_stats: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillCast {
    pub recorded_on: i64,
    pub last_recorded_on: i64,
    pub hits: Vec<SkillHit>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillHit {
    pub recorded_on: i64,
    pub damage: i64,
    pub crit: bool,
    pub back_attack: bool,
    pub front_attack: bool,
    pub buffed_by: Vec<u32>,
    pub debuffed_by: Vec<u32>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
}

#[derive(Debug)]
pub struct DamageData {
    pub is_initial: bool,
    pub skill_id: Option<u32>,
    pub skill_effect_id: Option<u32>,
    pub damage: i64,
    pub hit_option: HitOption,
    pub hit_flag: HitFlag,
    pub target_current_hp: i64,
    pub target_max_hp: i64,
    pub recorded_on: DateTime<Utc>,
    pub source_id: u64
}

impl DamageData {
    pub fn from(
        is_initial: bool,
        boss_only_damage: bool,
        source_id: u64,
        target_id: u64,
        recorded_on: DateTime<Utc>,
        skill_id: Option<u32>,
        skill_effect_id: Option<u32>,
        data: SkillDamageEvent) -> Option<Self> {

        if(source_id == target_id) {
            return None
        }

        let hit_flag = data.modifier & 0xf;
        let hit_flag = unsafe { transmute::<u8, HitFlag>(hit_flag as u8) };
        let hit_flag = if hit_flag as u8 == 15 { HitFlag::Unknown } else { hit_flag };

        let hit_option = (data.modifier >> 4) & 0x7;
        let hit_option = unsafe { transmute::<u8, HitOption>(hit_option as u8) };
        let hit_option = if hit_option as u8 >= 4 { HitOption::None } else { hit_option };

        if hit_flag == HitFlag::Invincible {
            return None;
        }

        if hit_flag == HitFlag::DamageShare
            && skill_id.is_none()
            && skill_effect_id.is_none()
        {
            return None;
        }

        let data = DamageData {
            is_initial,
            skill_id,
            skill_effect_id,
            damage: data.damage + data.shield_damage.p64_0.unwrap_or_default(),
            hit_option,
            hit_flag,
            source_id,
            target_current_hp: data.cur_hp,
            target_max_hp: data.max_hp,
            recorded_on,
        };

        Some(data)
    }
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub gauge1: u32,
    pub gauge2: u32,
    pub gauge3: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IncapacitatedEvent {
    #[serde(rename = "type")]
    pub event_type: IncapacitationEventType,
    pub recorded_on: DateTime<Utc>,
    // in a live meter, this might be retroactively updated to be shortened if the user uses get up or gets incapacitated with the same type again
    pub duration: Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum IncapacitationEventType {
    FallDown,
    CrowdControl,
}

pub type IdentityLog = Vec<(i64, (u32, u32, u32))>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityArcanist {
    // timestamp, (percentage, card, card)
    pub log: Vec<(i32, (f32, u32, u32))>,
    pub average: f64,
    pub card_draws: HashMap<u32, u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityArtistBard {
    // timestamp, (percentage, bubble)
    pub log: Vec<(i32, (f32, u32))>,
    pub average: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityGeneric {
    // timestamp, percentage
    pub log: Vec<(i32, f32)>,
    pub average: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
#[serde_as]
pub struct EncounterMisc {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boss_hp_log: Option<HashMap<String, Vec<BossHpLog>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raid_clear: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_info: Option<HashMap<i32, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdps_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdps_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ntp_fight_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_save: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BossHpLog {
    pub time: i32,
    pub hp: i64,
    #[serde(default)]
    pub p: f32,
}

impl BossHpLog {
    pub fn new(time: i32, hp: i64, p: f32) -> Self {
        Self { time, hp, p }
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Npc {
    pub id: i32,
    pub name: Option<String>,
    pub grade: String,
    #[serde(rename = "type")]
    pub npc_type: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Esther {
    pub name: String,
    pub icon: String,
    pub skills: Vec<i32>,
    #[serde(alias = "npcs")]
    pub npc_ids: Vec<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillData {
    pub id: i32,
    pub name: Option<String>,
    #[serde(rename = "type", default)]
    pub skill_type: String,
    pub desc: Option<String>,
    pub class_id: u32,
    pub icon: Option<String>,
    pub identity_category: Option<String>,
    #[serde(alias = "groups")]
    pub groups: Option<Vec<i32>>,
    pub summon_source_skills: Option<Vec<u32>>,
    pub source_skills: Option<Vec<u32>>,
    #[serde(default)]
    pub is_hyper_awakening: bool,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillEffectData {
    pub id: i32,
    pub comment: String,
    #[serde(skip)]
    pub stagger: i32,
    pub source_skills: Option<Vec<u32>>,
    pub directional_mask: Option<i32>,
    pub item_name: Option<String>,
    pub item_desc: Option<String>,
    pub item_type: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillBuffData {
    pub id: i32,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub icon: Option<String>,
    pub icon_show_type: Option<String>,
    pub duration: i32,
    // buff | debuff
    pub category: String,
    #[serde(rename(deserialize = "type"))]
    #[serde(deserialize_with = "int_or_string_as_string")]
    pub buff_type: String,
    pub status_effect_values: Option<Vec<i32>>,
    pub buff_category: Option<String>,
    pub target: String,
    pub unique_group: u32,
    #[serde(rename(deserialize = "overlap"))]
    pub overlap_flag: i32,
    pub passive_options: Vec<PassiveOption>,
    pub source_skills: Option<Vec<u32>>,
    pub set_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PassiveOption {
    #[serde(rename(deserialize = "type"))]
    pub option_type: String,
    pub key_stat: String,
    pub key_index: i32,
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffect {
    pub target: StatusEffectTarget,
    pub category: String,
    pub buff_category: String,
    pub buff_type: u32,
    pub unique_group: u32,
    pub source: StatusEffectSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum StatusEffectTarget {
    #[default]
    OTHER,
    PARTY,
    SELF,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffectSource {
    pub name: String,
    pub desc: String,
    pub icon: String,
    pub skill: Option<SkillData>,
    pub set_name: Option<String>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct StatusEffectBuffTypeFlags: u32 {
        const NONE = 0;
        const DMG = 1;
        const CRIT = 1 << 1;
        const ATKSPEED = 1 << 2;
        const MOVESPEED = 1 << 3;
        const HP = 1 << 4;
        const DEFENSE = 1 << 5;
        const RESOURCE = 1 << 6;
        const COOLDOWN = 1 << 7;
        const STAGGER = 1 << 8;
        const SHIELD = 1 << 9;

        const ANY = 1 << 20;
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectData {
    pub effects: Vec<CombatEffectDetail>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectDetail {
    pub ratio: i32,
    pub cooldown: i32,
    pub conditions: Vec<CombatEffectCondition>,
    pub actions: Vec<CombatEffectAction>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CombatEffectCondition {
    #[serde(rename(deserialize = "type"))]
    pub condition_type: String,
    pub actor_type: String,
    pub arg: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CombatEffectAction {
    pub action_type: String,
    pub actor_type: String,
    pub args: Vec<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillFeatureOption {
    #[serde(rename(deserialize = "type"))]
    pub effect_type: String,
    pub level: u16,
    #[serde(rename(deserialize = "paramtype"))]
    pub param_type: String,
    pub param: Vec<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct EngravingData {
    pub id: u32,
    pub name: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterPreview {
    pub id: i32,
    pub fight_start: i64,
    pub boss_name: String,
    pub duration: i64,
    pub classes: Vec<i32>,
    pub names: Vec<String>,
    pub difficulty: Option<String>,
    pub local_player: String,
    pub my_dps: i64,
    pub favorite: bool,
    pub cleared: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncountersOverview {
    pub encounters: Vec<EncounterPreview>,
    pub total_encounters: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchFilter {
    pub bosses: Vec<String>,
    pub min_duration: i32,
    pub max_duration: i32,
    pub cleared: bool,
    pub favorite: bool,
    pub difficulty: String,
    pub boss_only_damage: bool,
    pub sort: String,
    pub order: String,
}


#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDbInfo {
    pub size: String,
    pub total_encounters: i32,
    pub total_encounters_filtered: i32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum HitOption {
    None = 0,
    BackAttack = 1,
    FrontalAttack = 2,
    FlankAttack = 3,
    Max = 4,
}

impl HitOption {
    pub fn from() {

    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum HitFlag {
    Normal = 0,
    Critical = 1,
    Miss = 2,
    Invincible = 3,
    Dot = 4,
    Immune = 5,
    ImmuneSilenced = 6,
    FontSilenced = 7,
    DotCritical = 8,
    Dodge = 9,
    Reflect = 10,
    DamageShare = 11,
    DodgeHit = 12,
    Max = 13,
    Unknown = 14
}

impl HitFlag {
    pub fn from() {
        
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalInfo {
    pub client_id: String,
    pub updated_on: DateTime<Utc>,
    pub local_players: HashMap<u64, LocalPlayer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalPlayer {
    pub name: String,
    pub count: i32,
}

fn default_true() -> bool {
    true
}

fn int_or_string_as_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(serde::de::Error::custom("Expected a string or an integer")),
    }
}

#[derive(Debug, Serialize)]
pub struct OngoingEncounter {
    pub is_valid: bool,
    pub encounter: Encounter,
    pub party_info: HashMap<i32, Vec<String>>
}

#[derive(Debug, Default, Clone, Copy, AsRefStr, PartialEq, EnumString, VariantNames)]
#[repr(u8)]
pub enum RaidDifficulty {
    #[default]
    Unknown = 127,
    Normal = 0,
    Hard = 1,
    Inferno = 2,
    Challenge = 3,
    Solo = 4,
    #[strum(serialize = "The First")]
    TheFirst = 5,
    Trial = 6
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectTargetType {
    #[default]
    Party = 0,
    Local = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectCategory {
    #[default]
    Other = 0,
    Debuff = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectBuffCategory {
    #[default]
    Other = 0,
    Bracelet = 1,
    Etc = 2,
    BattleItem = 3,
    Elixir = 4,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectShowType {
    #[default]
    Other = 0,
    All = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectType {
    #[default]
    Shield = 0,
    Other = 1,
    HardCrowdControl = 2, // stun, root, MC, etc
}

#[derive(Debug, Default, Clone)]
pub struct StatusEffectDetails {
    pub instance_id: u32,
    pub status_effect_id: u32,
    pub custom_id: u32,
    pub target_id: u64,
    pub source_id: u64,
    pub target_type: StatusEffectTargetType,
    pub db_target_type: String,
    pub value: u64,
    pub stack_count: u8,
    pub category: StatusEffectCategory,
    pub buff_category: StatusEffectBuffCategory,
    pub show_type: StatusEffectShowType,
    pub status_effect_type: StatusEffectType,
    pub expiration_delay: f32,
    pub expire_at: Option<DateTime<Utc>>,
    pub end_tick: u64,
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub source_skills: Vec<u32>
}

impl StatusEffectDetails {
    /// infinite if duration is (sub-)zero or longer than an hour
    pub fn is_infinite(&self) -> bool {
        self.expiration_delay <= 0.0 || self.expiration_delay > 3600.0
    }
}
