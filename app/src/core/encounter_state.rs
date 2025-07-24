use chrono::Duration;
use chrono::{DateTime, Utc};
use hashbrown::{HashMap, HashSet};
use log::*;
use meter_core::packets::common::SkillMoveOptionData;
use meter_core::packets::definitions::*;
use meter_core::packets::structures::*;
use moka::sync::Cache;
use rsntp::SntpClient;
use tokio::task;
use std::cmp::max;
use std::collections::BTreeMap;
use std::default::Default;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::constants::WORKSHOP_BUFF_ID;
use crate::constants::{DB_NAME, TIMEOUT_DELAY_MS};
use crate::misc::data::*;
use crate::database::SaveToDb;
use crate::entity::npc::Boss;
use crate::entity::player::{self, Player, PlayerStats};
use crate::entity::{Entity, EntityVariant};
use crate::models::*;
use crate::models::TripodIndex;
use crate::models::TripodLevel;
use crate::core::utils::*;

pub type StatusEffectRegistry = HashMap<u32, StatusEffectDetails>;

#[derive(Debug)]
pub struct Local {
    id: u64,
    character_id: u64,
    name: String
}

#[derive(Debug)]
pub struct CurrentBoss {
    id: u64,
    name: Rc<String>
}

#[derive(Debug)]
pub struct PlayerSlim {
    id: u64,
    party_id: u32,
    name: String,
}

#[derive(Debug, Default)]
pub struct Raid {
    pub id: u32,
    pub updated_on: DateTime<Utc>,
    pub parties: HashMap<u32, Party>,
}

#[derive(Debug, Default)]
pub struct Party {
    pub id: u32,
    pub members: Vec<PlayerSlim>,
}

#[derive(Debug)]
pub struct EncounterState {
    version: String,
    raid: Raid,
    ignore_damage_timeout: Duration,
    pub update_interval: Duration,
    last_party_update: std::time::Instant,
    party_duration: std::time::Duration,
    entities: HashMap<u64, Entity>,
    pub local: Local,
    pub is_resetting: bool,
    pub boss_dead_update: bool,
    pub saved: bool,
    pub party_freeze: bool,
    pub raid_clear: bool,
    pub valid_zone: bool,
    damage_log: HashMap<u64, Vec<(i64, i64)>>,
    cast_log: HashMap<u64, HashMap<u32, Vec<i32>>>,
    boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    pub party_info: Vec<Vec<String>>,
    pub raid_difficulty: RaidDifficulty,
    pub boss_only_damage: bool,
    pub region: Option<String>,
    sntp_client: SntpClient,
    ntp_fight_start: i64,
    pub rdps_valid: bool,
    custom_id_map: HashMap<u32, u32>,
    pub raid_end_cd: DateTime<Utc>,
    pub damage_is_valid: bool,
    entity_id_to_party_id: HashMap<u64, u32>,
    players_by_character_id: HashMap<u64, PlayerSlim>,
    local_status_effect_registry: HashMap<u64, StatusEffectRegistry>,
    party_status_effect_registry: HashMap<u64, StatusEffectRegistry>,
    pub started_on: DateTime<Utc>,
    updated_on: DateTime<Utc>,
    sent_on: DateTime<Utc>,
    pub skills: HashMap<(u64, u32, i64), SkillCast>,
    pub projectile_id_to_timestamp: Cache<u64, i64>,
    pub skill_timestamp: Cache<(u64, u32), i64>,
    pub damage_stats: EncounterDamageStats,
    pub current_boss: CurrentBoss,
}

impl EncounterState {
    pub fn new(version: String) -> EncounterState {
        EncounterState {
            version,
            entity_id_to_party_id: HashMap::new(),
            raid: Default::default(),
            players_by_character_id: HashMap::new(),
            ignore_damage_timeout: Duration::seconds(10),
            update_interval: Duration::milliseconds(200),
            last_party_update: std::time::Instant::now(),
            party_duration: std::time::Duration::from_millis(2000),
            local: Local { 
                id: 0,
                character_id: 0,
                name: "".into()
            },
            started_on: DateTime::<Utc>::MIN_UTC,
            updated_on: DateTime::<Utc>::MIN_UTC,
            sent_on: DateTime::<Utc>::MIN_UTC,
            skills: HashMap::new(),
            projectile_id_to_timestamp: Cache::builder()
                .time_to_idle(std::time::Duration::from_secs(20))
                .build(),
            skill_timestamp: Cache::builder()
                .time_to_idle(std::time::Duration::from_secs(20))
                .build(),
            local_status_effect_registry: HashMap::new(),
            party_status_effect_registry: HashMap::new(),
            entities: HashMap::new(),
            raid_end_cd: DateTime::<Utc>::MIN_UTC,
            valid_zone: true,
            party_freeze: false,
            is_resetting: false,
            raid_clear: false,
            boss_dead_update: false,
            saved: false,
            damage_log: HashMap::new(),
            boss_hp_log: HashMap::new(),
            cast_log: HashMap::new(),
            party_info: Vec::new(),
            raid_difficulty: RaidDifficulty::Unknown,
            boss_only_damage: false,
            region: None,
            sntp_client: SntpClient::new(),
            ntp_fight_start: 0,
            rdps_valid: false,
            custom_id_map: HashMap::new(),
            damage_is_valid: true,
            damage_stats: EncounterDamageStats::default(),
            current_boss: CurrentBoss { 
                id: 0,
                name: String::from("").into()
            }
        }
    }

    pub fn get_ongoing_encounter(&mut self, now: DateTime<Utc>) -> Option<OngoingEncounter> {

        let boss_dead_update = self.boss_dead_update;

        if self.boss_dead_update {
            self.boss_dead_update = false;
        }

        let can_send = self.sent_on == DateTime::<Utc>::MIN_UTC
            || self.sent_on - now > self.update_interval
            || self.is_resetting
            || self.boss_dead_update;

        if !can_send {
            return None
        }

        self.sent_on = now;

        let current_boss = self.entities.get(&self.current_boss.id)
            .and_then(|pr| pr.as_boss()).map(|pr| {
            EncounterEntity {
                id: pr.id,

                ..Default::default()
            }
        });

        let entities = self.entities.iter().filter_map(|(key, value)| {
            match value.deref() {
                EntityVariant::Unknown => None,
                EntityVariant::Projectile(projectile) => None,
                EntityVariant::Player(player) => {
                    if player.encounter_stats.damage_stats.dealt == 0 {
                        return None
                    }

                    Some((player.name.clone(), EncounterEntity {
                        id: *key,
                        name: player.name.clone(),
                        character_id: player.character_id,
                        class_id: player.class_id,
                        entity_type: EntityType::Player,
                        damage_stats: DamageStats {
                            damage_absorbed: player.encounter_stats.damage_stats.absorbed,
                            damage_dealt: player.encounter_stats.damage_stats.dealt,
                            ..Default::default()
                        },
                        ..Default::default()
                    }))
                },
                EntityVariant::Npc(npc) => None,
                EntityVariant::Boss(boss) => {
                    if boss.encounter_stats.dealt == 0 {
                        return None
                    }

                    Some((boss.name.to_string(), EncounterEntity {
                        id: *key,
                        name: boss.name.to_string(),
                        entity_type: EntityType::Boss,
                        damage_stats: DamageStats {
                            damage_dealt: boss.encounter_stats.dealt,
                            ..Default::default()
                        },
                        ..Default::default()
                    }))
                },
                EntityVariant::Esther(esther) => {
                    if esther.damage_dealt == 0 {
                        return None
                    }

                    Some((esther.name.clone(), EncounterEntity {
                        id: *key,
                        name: esther.name.clone(),
                        entity_type: EntityType::Esther,
                        damage_stats: DamageStats {
                            damage_dealt: esther.damage_dealt,
                            ..Default::default()
                        },
                        ..Default::default()
                    }))
                },
            }

        }).collect();

        let result = OngoingEncounter {
            is_valid: self.damage_is_valid,
            party_info: HashMap::new(),
            encounter: Encounter {
                last_combat_packet: self.updated_on.timestamp_millis(),
                fight_start: self.started_on.timestamp_millis(),
                local_player: self.local.name.clone(),
                entities,
                current_boss_name: self.current_boss.name.to_string(),
                current_boss,
                encounter_damage_stats: self.damage_stats.clone(),
                duration: 0,
                difficulty: Some(self.raid_difficulty.as_ref().to_string()), 
                favorite: false,
                cleared: false,
                boss_only_damage: self.boss_only_damage,
                sync: None,
                region: self.region.clone()
            },
        };

        Some(result)
    }

    pub fn get_encounter(&self, is_manual: bool) -> Option<SaveToDb> {

        if !is_manual && self.current_boss.name.is_empty() {
            return None
        }

        if !is_manual && !self.entities.contains_key(&self.current_boss.id) {
            return None
        }

        if !is_manual && !self.entities
            .values()
            .filter_map(|entity| entity.as_player())
            .any(|pr| pr.encounter_stats.damage_stats.dealt > 0) {
            return None
        }

        let duration = self.updated_on - self.started_on;
    
        let entities: Vec<EncounterEntity> = self.entities.values().filter_map(|pr | {
            match pr.deref() {
                EntityVariant::Player(player) => {

                    if player.class_id == 0 
                        || player.encounter_stats.damage_stats.dealt == 0
                        && !player.is_local {
                        return None
                    }

                    Some(EncounterEntity {
                        id: player.id,
                        name: player.name.clone(),
                        character_id: player.character_id,
                        class_id: player.class_id,
                        entity_type: EntityType::Player,
                        damage_stats: DamageStats {
                            ..Default::default()    
                        },
                        ..Default::default()
                    })
                },
                EntityVariant::Boss(boss) => {

                    if boss.encounter_stats.max_hp == 0 
                        || boss.encounter_stats.dealt == 0 {
                        return None
                    }

                    Some(EncounterEntity {
                        id: boss.id,
                        name: boss.name.to_string(),
                        entity_type: EntityType::Boss,
                        damage_stats: DamageStats {
                            ..Default::default()    
                        },
                        ..Default::default()
                    })
                },
                EntityVariant::Esther(esther) => {

                    if esther.damage_dealt == 0 {
                        return None
                    }

                    Some(EncounterEntity {
                        id: esther.id,
                        npc_id: esther.npc_id,
                        name: esther.name.clone(),
                        entity_type: EntityType::Esther,
                        ..Default::default()
                    })
                },
                _ => None
            }
            
        }).collect();

        let boss_max_hp = self.entities
            .get(&self.current_boss.id)
            .and_then(|pr| pr.as_boss())
            .unwrap()
            .encounter_stats.max_hp;

        let duration_seconds = max(duration.num_seconds() / 1000, 1);
        let dps = self.damage_stats.total_damage_dealt / duration_seconds;
        let skill_cast_log = self.get_cast_log();
        let current_boss_name = get_main_boss_name(&self.current_boss.name);

        let misc: EncounterMisc = EncounterMisc {
            raid_clear: if self.raid_clear { Some(true) } else { None },
            party_info: if self.party_info.is_empty() {
                None
            } else {
                Some(
                    self.party_info
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(index, party)| (index as i32, party))
                        .collect(),
                )
            },
            region: self.region.clone(),
            version: Some(self.version.clone()),
            rdps_valid: Some(self.rdps_valid),
            rdps_message: if self.rdps_valid {
                None
            } else {
                Some("invalid_stats".to_string())
            },
            ntp_fight_start: Some(self.ntp_fight_start),
            manual_save: Some(is_manual),
            ..Default::default()
        };

        let model = SaveToDb {
            boss_only_damage: self.boss_only_damage,
            boss_max_hp,
            duration,
            duration_seconds,
            entities,
            current_boss_name: self.current_boss.name.to_string(),
            local_player: self.local.name.clone(),
            started_on: self.started_on,
            updated_on: self.updated_on,
            encounter_damage_stats: self.damage_stats.clone(),
            misc, 
            damage_log: self.damage_log.clone(),
            cast_log: self.cast_log.clone(),
            boss_hp_log: self.boss_hp_log.clone(),
            raid_clear: self.raid_clear,
            party_info: todo!(),
            player_info: todo!(),
            raid_difficulty: self.raid_difficulty,
            region: self.region,
            version: self.version,
            ntp_fight_start: 0,
            rdps_valid: self.rdps_valid,
            is_manual: is_manual,
            skill_cast_log
        };

        Some(model)
    }

    pub fn get_party(&self) -> Vec<Vec<String>> {

        let mut parties: Vec<&Party> = self.raid.parties.values().collect();
        parties.sort_by_key(|p| p.id);

        parties.iter().map(|party| {
            party.members
                .iter()
                .map(|player| player.name.clone())
                .collect()
        }).collect()
    }

    pub fn has_fight_started(&self) -> bool {
        self.started_on != DateTime::<Utc>::MIN_UTC
    }

    // use this to make sure damage packets are not tracked after a raid just wiped
    pub fn has_restarted(&self, now: DateTime<Utc>) -> bool {
        now - self.raid_end_cd < self.ignore_damage_timeout
    }

    pub fn get_entity_mut(&mut self, id: &u64) -> Option<&mut EntityVariant> {
        self.entities.get_mut(id).map(|pr| pr.deref_mut())
    }

    pub fn new_cast(
        &mut self,
        entity_id: u64,
        skill_id: u32,
        summon_source: Option<Vec<u32>>,
        recorded_on: DateTime<Utc>
    ) {
        let relative = recorded_on - self.started_on;
        let relative = relative.num_milliseconds();
        if let Some(summon_source) = summon_source {
            for source in summon_source {
                if self.skill_timestamp.get(&(entity_id, source)).is_some() {

                    return;
                }
            }
        }

        self.skill_timestamp.insert((entity_id, skill_id), relative);
        self.skills.insert(
            (entity_id, skill_id, relative),
            SkillCast {
                hits: Vec::new(),
                recorded_on: relative,
                last_recorded_on: relative,
            },
        );
    }

    pub fn on_hit(
        &mut self,
        entity_id: u64,
        projectile_id: u64,
        skill_id: u32,
        info: &SkillHit,
        summon_source: &Vec<u32>,
    ) {
        let skill_timestamp = if !summon_source.is_empty() {
            let mut source_timestamp = info.recorded_on;
            let mut found = false;
            for source in summon_source {
                let key = (entity_id, *source);
                if let Some(skill_timestamp) = self.skill_timestamp.get(&key) {
                    found = true;
                    source_timestamp = skill_timestamp;
                    break;
                }
            }

            if !found {
                self.skill_timestamp
                    .insert((entity_id, skill_id), source_timestamp);
            }

            source_timestamp

        } else if let Some(skill_timestamp) = self.projectile_id_to_timestamp.get(&projectile_id) {
            skill_timestamp
        } else if let Some(skill_timestamp) = self.skill_timestamp.get(&(entity_id, skill_id)) {
            skill_timestamp
        } else {
            -1
        };

        if skill_timestamp >= 0 {
            let recorded_on = info.recorded_on;
            self.skills
                .entry((entity_id, skill_id, skill_timestamp))
                .and_modify(|skill| {
                    skill.hits.push(info.clone());
                    skill.last_recorded_on = recorded_on;
                })
                .or_insert(SkillCast {
                    hits: vec![info.clone()],
                    recorded_on: recorded_on,
                    last_recorded_on: recorded_on,
                });
        }
    }

    pub fn get_cast_log(&self) -> HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> {
        let mut cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> = HashMap::new();
        for ((entity_id, skill_id, timestamp), cast) in self.skills.iter() {
            cast_log
                .entry(*entity_id)
                .or_default()
                .entry(*skill_id)
                .or_default()
                .insert(*timestamp, cast.clone());
        }

        cast_log
    }

    fn get_removed_shields(
        &mut self,
        target_type: StatusEffectTargetType,
        target_id: u64,
        instance_ids: Vec<u32>,
        reason: u8,
    ) -> Vec<StatusEffectDetails> {
        let registry = self.get_registry_mut(target_type, target_id);

        let mut removed_shields = Vec::new();

        if let Some(ser) = registry {
            for id in instance_ids {
                if let Some(status_effect) = ser.remove(&id) {
                    if status_effect.status_effect_type == StatusEffectType::Shield && reason == 4 {
                        removed_shields.push(status_effect);
                    }
                }
            }
        }

        removed_shields
    }

    pub fn on_party_status_effects_remove(
        &mut self,
        target_id: u64,
        instance_ids: Vec<u32>,
        reason: u8,
        recorded_on: DateTime<Utc>) {

        let shields = self.get_removed_shields(StatusEffectTargetType::Party, target_id, instance_ids, reason);
        
        for status_effect in shields {
            let source_id = status_effect.source_id;
            let character_id = status_effect.target_id;
            let buff_id = status_effect.status_effect_id;
            let value = status_effect.value;

            if value == 0 {
                return;
            }
            
            let source = Self::get_source_entity_unsafe(self, source_id, recorded_on);
            let target = Self::get_player_by_character_id_unsafe(self, character_id);

            match (source.deref_mut(), target) {
                (EntityVariant::Player(source), Some(target)) => {
                    self.damage_stats.total_effective_shielding += value;

                    target.encounter_stats.damage_stats.absorbed += value;
                    target.encounter_stats.damage_stats.absorbed_by.entry(buff_id)
                        .and_modify(|e| *e += value)
                        .or_insert(value);
                    source.encounter_stats.damage_stats.absorbed_on_others += value;
                    source.encounter_stats.damage_stats
                        .absorbed_on_others_by
                        .entry(buff_id)
                        .and_modify(|e| *e += value)
                        .or_insert(value);
                },
                _ => {}
            }
        }
    }

    pub fn get_player_by_character_id_unsafe<'a>(state: *mut EncounterState, id: u64) -> Option<&'a mut Player> {
        let player = unsafe { (*state).players_by_character_id
            .get(&id)
            .and_then(|pr| (*state).entities.get_mut(&pr.id))
            .and_then(|pr| pr.as_player_mut()) };

        player
    }

    pub fn get_player_by_character_id(&mut self, id: u64) -> Option<&mut Player> {
        let player = self.players_by_character_id
            .get(&id)
            .and_then(|pr| self.entities.get_mut(&pr.id))
            .and_then(|pr| pr.as_player_mut());

        player
    }

    pub fn remove_local_object(&mut self, id: u64) {
        self.local_status_effect_registry.remove(&id);
    }

    pub fn sync_status_effect(
        &mut self,
        instance_id: u32,
        character_id: u64,
        object_id: u64,
        value: u64,
        recorded_on: DateTime<Utc>
    ){
        let use_party = self.should_use_party_status_effect(character_id, self.local.character_id);
        
        let (target_id, target_type) = if use_party {
            (character_id, StatusEffectTargetType::Party)
        } else {
            (object_id, StatusEffectTargetType::Local)
        };
        
        if target_id == 0 {
            return
        }

        let self_ptr = self as *mut Self;
        let registry = unsafe { (*self_ptr).get_registry(target_type, target_id) };

        let ser = match registry {
            Some(ser) => ser,
            None => return,
        };

        let status_effect = match ser.get(&instance_id) {
            Some(se) => se,
            None => return,
        };

        if status_effect.status_effect_type == StatusEffectType::Shield {
            let source_id = status_effect.source_id;
            let buff_id = status_effect.status_effect_id;
            let change = status_effect.value
                .checked_sub(value)
                .unwrap_or_default();
            
            match target_type {
                StatusEffectTargetType::Party => {
                    let character_id = status_effect.target_id;
                    
                    let source = Self::get_source_entity_unsafe(self, source_id, recorded_on);
                    let target = Self::get_player_by_character_id_unsafe(self, target_id);

                    match (source.deref_mut(), target) {
                        (EntityVariant::Player(source), Some(target)) => {
                            self.damage_stats.total_effective_shielding += value;

                            target.encounter_stats.damage_stats.absorbed += value;
                            target.encounter_stats.damage_stats.absorbed_by.entry(buff_id)
                                .and_modify(|e| *e += value)
                                .or_insert(value);
                            source.encounter_stats.damage_stats.absorbed_on_others += value;
                            source.encounter_stats.damage_stats
                                .absorbed_on_others_by
                                .entry(buff_id)
                                .and_modify(|e| *e += value)
                                .or_insert(value);
                        },
                        _ => {}
                    }
                },
                StatusEffectTargetType::Local => {
                    let target_id = status_effect.target_id;

                    let source = Self::get_source_entity_unsafe(self, source_id, recorded_on);
                    let target = Self::get_source_entity_unsafe(self, target_id, recorded_on);

                    match (source.deref_mut(), target.deref_mut()) {
                        (EntityVariant::Player(source), EntityVariant::Player(target)) => {
                            self.damage_stats.total_effective_shielding += value;

                            target.encounter_stats.damage_stats.absorbed += value;
                            target.encounter_stats.damage_stats.absorbed_by.entry(buff_id)
                                .and_modify(|e| *e += value)
                                .or_insert(value);
                            source.encounter_stats.damage_stats.absorbed_on_others += value;
                            source.encounter_stats.damage_stats
                                .absorbed_on_others_by
                                .entry(buff_id)
                                .and_modify(|e| *e += value)
                        .or_insert(value);
                        },
                        (_, EntityVariant::Boss(target)) => {
                            target.encounter_stats.current_shield = value;
                        },
                        _ => {}
                    }
                },
            }
        }
    }

    pub fn get_registry_mut(
        &mut self,
        target_type: StatusEffectTargetType,
        target_id: u64
    ) -> Option<&mut HashMap<u32, StatusEffectDetails>> {
        let registry = match target_type {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        registry.get_mut(&target_id)
    }

    pub fn get_registry(
        &self,
        target_type: StatusEffectTargetType,
        target_id: u64
    ) -> Option<&HashMap<u32, StatusEffectDetails>> {
        let registry = match target_type {
            StatusEffectTargetType::Local => &self.local_status_effect_registry,
            StatusEffectTargetType::Party => &self.party_status_effect_registry,
        };

        registry.get(&target_id)
    }

    pub unsafe fn get_status_effects_player_vs_boss(
        &mut self,
        timestamp: DateTime<Utc>,
        entity_id: u64,
        character_id: u64,
        target_id: u64,
    ) -> (Vec<&StatusEffectDetails>, Vec<&StatusEffectDetails>) {

        let use_party_for_source = self.should_use_party_status_effect(character_id, self.local.character_id);

        let (source_id, source_type) = if use_party_for_source {
            (character_id, StatusEffectTargetType::Party)
        } else {
            (entity_id, StatusEffectTargetType::Local)
        };

        let self_ptr = self as *mut Self;
        let status_effects_on_source =
            self.actually_get_status_effects(source_id, source_type, timestamp);

        let source_party_id = (*self_ptr).entity_id_to_party_id.get(&entity_id);
        
        let mut status_effects_on_target = match source_party_id {
            Some(&source_party_id) => (*self_ptr).get_status_effects_from_party(
                target_id,
                StatusEffectTargetType::Local,
                source_party_id,
                timestamp,
            ),
            None => (*self_ptr).actually_get_status_effects(
                target_id,
                StatusEffectTargetType::Local,
                timestamp,
            ),
        };

        status_effects_on_target.retain(|se| {
            !(se.target_type == StatusEffectTargetType::Local
                && se.category == StatusEffectCategory::Debuff
                && se.source_id != source_id
                && se.db_target_type == "self")
        });

        (status_effects_on_source, status_effects_on_target)
    }

    pub fn actually_get_status_effects(
        &mut self,
        target_id: u64,
        target_type: StatusEffectTargetType,
        timestamp: DateTime<Utc>,
    ) -> Vec<&StatusEffectDetails> {
        let registry = self.get_registry_mut(target_type, target_id);
        
        let ser = match registry {
            Some(ser) => ser,
            None => return Vec::new(),
        };

        ser.retain(|_, se| se.expire_at.map_or(true, |expire_at| expire_at > timestamp));
        let values = ser.values().collect();
        values
    }

    pub fn get_status_effects_from_party(
        &mut self,
        target_id: u64,
        target_type: StatusEffectTargetType,
        party_id: u32,
        timestamp: DateTime<Utc>,
    ) -> Vec<&StatusEffectDetails> {
        let selt_ptr = self as *mut Self;
        let registry = unsafe { (*selt_ptr).get_registry_mut(target_type, target_id) };

        let ser = match registry {
            Some(ser) => ser,
            None => return Vec::new(),
        };

        ser.retain(|_, se| se.expire_at.map_or(true, |expire_at| expire_at > timestamp));

        let entity_id_to_party_id = unsafe { &(*selt_ptr).entity_id_to_party_id };

        ser.values()
            .filter(|x| {
                is_valid_for_raid(x)
                    || entity_id_to_party_id.get(&x.source_id).filter(|pr| **pr == party_id).is_some()
            })
            .collect()
    }

    fn should_use_party_status_effect(&self, character_id: u64, local_character_id: u64) -> bool {

        let [player, local] = [
            self.players_by_character_id.get(&character_id),
            self.players_by_character_id.get(&local_character_id)
        ];

        match (player, local) {
            (Some(player), Some(local)) => player.party_id == local.party_id,
            _ => false
        }
    }

    pub fn clear(&mut self) {
        self.local_status_effect_registry.clear();
        self.party_status_effect_registry.clear();
    }

    pub fn init_env(&mut self, now: DateTime<Utc>, id: u64) {
        
        let mut local_player = self
            .entities
            .remove(&self.local.id)
            .unwrap_or_else(|| Entity::unknown_local(id, Utc::now()));

        info!("init env: eid: {}->{}", self.local.id, id);

        self.local.id = id;
        self.entities.clear();
        
        self.entities.insert(id, local_player);
        self.clear();
    }

    pub fn init_pc(
        &mut self,
        created_on: DateTime<Utc>,
        id: u64,
        name: String,
        character_id: u64,
        class_id: u32,
        max_item_level: f32,
        stat_pairs: Vec<StatPair>,
        status_effect_datas: Vec<StatusEffectData>,
        current_hp: i64,
        max_hp: i64) {
        let entity = Player {
            id,
            name: name.clone(),
            class_id,
            character_id,
            encounter_stats: PlayerStats {
                max_hp,
                current_hp,
                ..Default::default()
            },
            is_local: true,
            gear_level: truncate_gear_level(max_item_level),
            game_stats: stat_pairs
                .iter()
                .map(|sp: &StatPair| (sp.stat_type, sp.value))
                .collect(),
            incapacitations: vec![]
        };
        let entity = Entity::player(id, entity, created_on);

        self.local.id = id;
        self.local.name = name;
        self.local.character_id = character_id;

        self.entities.clear();
        self.entities.insert(id, entity);

        self.local_status_effect_registry.remove(&id);
        
        for sed in status_effect_datas.into_iter() {
            self.build_and_register_status_effect(&sed, id, created_on);
        }

    }


    pub fn new_pc(
        &mut self,
        now: DateTime<Utc>,
        id: u64,
        name: String,
        character_id: u64,
        class_id: u32,
        max_item_level: f32,
        current_hp: i64,
        max_hp: i64,
        stat_pairs: Vec<StatPair>,
        status_effect_datas: Vec<StatusEffectData>
    ) {
        let entity = Player {
            id, 
            name,
            class_id,
            character_id,
            encounter_stats: PlayerStats {
                max_hp,
                current_hp,
                ..Default::default()
            },
            is_local: false,
            gear_level: truncate_gear_level(max_item_level),
            game_stats: stat_pairs
                .iter()
                .map(|sp| (sp.stat_type, sp.value))
                .collect(),
            incapacitations: vec![]
        };
        let entity = Entity::player(id, entity, now);

        let use_party_status_effects =
            self.should_use_party_status_effect(character_id, self.local.character_id);
        
        if use_party_status_effects {
            self.party_status_effect_registry.remove(&character_id);
        } else {
            self.local_status_effect_registry.remove(&character_id);
        }

        let (target_id, target_type) = if use_party_status_effects {
            (character_id, StatusEffectTargetType::Party)
        } else {
            (id, StatusEffectTargetType::Local)
        };

        for sed in status_effect_datas.into_iter() {
            let source_id = sed.source_id;
            let status_effect = build_status_effect(sed, target_id, source_id, target_type, now);

            let registry = match target_type {
                StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
                StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
            };

            let sub_registry = registry.entry(target_id).or_insert_with(HashMap::new);

            sub_registry.insert(status_effect.instance_id, status_effect);
        }

        info!("{entity}");
        self.entities.insert(id, entity);
    }

    pub fn new_npc(
        &mut self,
        created_on: DateTime<Utc>,
        id: u64,
        type_id: u32,
        level: u16,
        balance_level: u16,
        max_hp: i64,
        stat_pairs: Vec<StatPair>,
        status_effect_datas: Vec<StatusEffectData>,
        ) {        
        let npc = Entity::npc(id, type_id, None, level, balance_level, max_hp, stat_pairs, created_on);

        if let EntityVariant::Boss(boss) = npc.deref() {
            self.current_boss.id = boss.id;
            self.current_boss.name = boss.name.clone();
        }

        self.entities.insert(id, npc);
        self.local_status_effect_registry.remove(&id);

        for sed in status_effect_datas.into_iter() {
            self.build_and_register_status_effect(&sed, id, created_on);
        }
    }

    pub fn new_npc_summon(
        &mut self,
        created_on: DateTime<Utc>,
        id: u64,
        npc_id: u32,
        owner_id: u64,
        level: u16,
        balance_level: u16,
        max_hp: i64,
        stat_pairs: Vec<StatPair>,
        status_effect_datas: Vec<StatusEffectData>
    ) {
        let entity = Entity::npc(
            id,
            npc_id,
            Some(owner_id),
            level,
            balance_level,
            max_hp,
            stat_pairs,
            created_on);

        self.entities.insert(id, entity);
        self.local_status_effect_registry.remove(&id);

        for sed in status_effect_datas.into_iter() {
            self.build_and_register_status_effect(&sed, id, created_on);
        }
    }

    pub unsafe fn party_status_effect_add(
        &mut self,
        now: DateTime<Utc>,
        character_id: u64,
        status_effect_datas: Vec<StatusEffectData>) {

        let mut shields: Vec<StatusEffectDetails> = Vec::new();
        let self_ptr = self as *mut Self;

        for sed in status_effect_datas {
            let entity = self.get_source_entity(sed.source_id, now);

            let mut status_effect = build_status_effect(
                sed,
                character_id,
                entity.id(),
                StatusEffectTargetType::Party,
                now,
            );

            if let Some(source) = entity.as_player_mut() {
                
                let custom_id = Self::get_custom_id(
                    status_effect.status_effect_id,
                    &status_effect.source_skills,
                    &source.encounter_stats.skills);
                if let Some(custom_id) = custom_id {
                    status_effect.custom_id = custom_id;
                    (*self_ptr).custom_id_map.insert(custom_id, status_effect.status_effect_id);
                }

                if status_effect.status_effect_type == StatusEffectType::Shield {
                    if let Some(target) = (*self_ptr).get_player_by_character_id(character_id) {
                        let buff_id = status_effect.status_effect_id;
                        let value = status_effect.value;
                        (*self_ptr).update_shielding_stats(buff_id, status_effect.value);

                        source.encounter_stats.shields_given += value;
                        source.encounter_stats.shields_given_by
                            .entry(buff_id)
                            .and_modify(|e| *e += value)
                            .or_insert(value);
                        target.encounter_stats.shields_received += value;
                        target.encounter_stats.shields_received_by
                            .entry(buff_id)
                            .and_modify(|e| *e += value)
                            .or_insert(value);
                    }
                }
            }

            let registry = &mut self.party_status_effect_registry;
            let sub_registry = registry.entry(status_effect.target_id).or_insert_with(HashMap::new);

            sub_registry.insert(status_effect.instance_id, status_effect);
        }
    }

    pub fn get_custom_id(status_effect_id: u32, source_skills: &[u32], player_skills: &HashMap<u32, Skill>) -> Option<u32> {
        if source_skills.len() <= 2 {
           return None
        }

        let mut updated_on = DateTime::<Utc>::MIN_UTC;
        let mut last_skill_id = 0_u32;

        for skill_id in source_skills {
            let skill_stat = player_skills.get(skill_id);

            if let Some(skill_stat) = skill_stat {
                if skill_stat.name.is_empty() {
                    continue;
                }

                if skill_stat.id == 21090 {
                    if let Some(tripods) = skill_stat.tripod_index {
                        if tripods.second != 2 {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if skill_stat.updated_on > updated_on {
                    last_skill_id = *skill_id;
                    updated_on = skill_stat.updated_on;
                }
            }
        }

        if last_skill_id > 0 {
            let custom_id = get_new_id(last_skill_id + status_effect_id);
            return Some(custom_id)
        }

        None
    }

    pub fn new_projectile(
        &mut self,
        id: u64,
        owner_id: u64,
        skill_id: u32,
        skill_effect_id: u32,
        created_on: DateTime<Utc>
    ) {
        let is_attack_battle_item = is_battle_item(&skill_effect_id, "attack");
        let entity = Entity::projectile(
            id,
            owner_id,
            is_attack_battle_item,
            skill_id,
            skill_effect_id,
            created_on);

        self.entities.insert(id, entity);
    }

    pub fn new_trap(
        &mut self,
        id: u64,
        owner_id: u64,
        skill_id: u32,
        skill_effect: u32,
        created_on: DateTime<Utc>
    ) {
        let entity = Entity::trap(id, owner_id, skill_id, skill_effect, created_on);
        self.entities.insert(id, entity);
    }

    pub fn party_info(
        &mut self,
        party_instance_id: u32,
        raid_instance_id: u32,
        party_member_datas: Vec<PKTPartyInfoInner>,
        local_players: &HashMap<u64, LocalPlayer>,
        recorded_on: DateTime<Utc>
    ) {
        let mut unknown_local = self.entities
            .get(&self.local.id)
            .and_then(|pr| pr.as_player())
            .map(|pr| pr.name.is_empty())
            .unwrap_or_else(|| true);
        
        let local_character_id = if unknown_local {
            let party_members: HashSet<u64> = party_member_datas
                .iter()
                .map(|m| m.character_id)
                .collect();

            local_players.keys().find(|&pr| party_members.contains(pr))
        } else { None };

        let player_id_by_character_id: HashMap<u64, u64> = self
            .entities
            .values()
            .filter_map(|pr| pr.as_player())
            .map(|pr| (pr.character_id, pr.id))
            .collect();

        self.raid.id = raid_instance_id;
        self.raid.updated_on = recorded_on;
        let party = self.raid.parties.entry(party_instance_id).or_default();
        party.id = party_instance_id;

        if !party.members.is_empty() {
            party.members.clear();
        }

        for member in &party_member_datas {
            party.members.push(PlayerSlim { id: member.character_id, party_id: party_instance_id, name: member.name.clone() });
        }

        for member in party_member_datas {
            let character_id = member.character_id;

            if unknown_local && local_character_id.filter(|&&pr| pr == character_id).is_some() {
                if let Some(local_player) = self.get_player_by_id(self.local.id) {
                    unknown_local = false;
                    local_player.class_id = member.class_id as u32;
                    local_player.gear_level = truncate_gear_level(member.gear_level);
                    local_player.name = member.name.clone();
                    local_player.character_id = character_id;
                }
            }

            if let Some(player) = self.get_player_by_character_id(character_id) {
                player.gear_level = truncate_gear_level(member.gear_level);
                player.class_id = member.class_id as u32;
            }

            match self.players_by_character_id.get_mut(&character_id) {
                Some(player) => {

                    player.id = player_id_by_character_id.get(&character_id).cloned().unwrap_or_default();
                    player.party_id = party_instance_id;

                    if player.id > 0 {
                        self.entity_id_to_party_id.insert(player.id, party_instance_id);
                    }
                },
                None => {
                    let player = PlayerSlim {
                        id: player_id_by_character_id.get(&character_id).cloned().unwrap_or_default(),
                        party_id: party_instance_id,
                        name: member.name
                    };

                    if player.id > 0 {
                        self.entity_id_to_party_id.insert(player.id, party_instance_id);
                    }

                    self.players_by_character_id.insert(character_id, player);
                },
            }
        }

    }

    pub fn is_initial(&self) -> bool {
        self.started_on == DateTime::<Utc>::MIN_UTC
    }

    // need to hard code clown because it spawns before the trigger is sent???
    pub fn is_saydon_glitch(&self) -> bool {
        self.current_boss.name.is_empty()
            || self.started_on == DateTime::<Utc>::MIN_UTC
            || self.current_boss.name.as_str() == "Saydon"
    }

    pub fn update_shielding_stats(&mut self, buff_id: u32, shield: u64) {
        if !self.damage_stats.applied_shield_buffs.contains_key(&buff_id)
        {
            let mut source_id: Option<u32> = None;
            let original_buff_id = if let Some(deref_id) = self.custom_id_map.get(&buff_id) {
                source_id = Some(get_skill_id(buff_id, *deref_id));
                *deref_id
            } else {
                buff_id
            };

            if let Some(status_effect) = get_status_effect_data(original_buff_id, source_id) {
                self.damage_stats.applied_shield_buffs.insert(buff_id, status_effect);
            }
        }

        self.damage_stats.total_shielding += shield;
    }

    pub fn get_source_entity(&mut self, id: u64, now: DateTime<Utc>) -> &mut Entity {
        
        let owner_id = self.entities.get(&id).and_then(|e| e.get_owner());
        let key = owner_id.unwrap_or(id);
        self.entities.entry(key).or_insert_with(|| Entity::unknown(id, now))
    }

    pub fn get_source_entity_unsafe<'a>(state: *mut EncounterState, id: u64, now: DateTime<Utc>) -> &'a mut Entity {
        let owner_id = unsafe { (*state).entities.get(&id).and_then(|e| e.get_owner()) };
        let key = owner_id.unwrap_or(id);
        unsafe { (*state).entities.entry(key).or_insert_with(|| Entity::unknown(id, now)) }
    }

    pub fn is_player(&mut self, id: u64) -> bool {
        self.entities.get(&id).and_then(|pr| pr.as_player()).is_some()
    }

    pub fn on_remove_objects(&mut self, ids: Vec<u64>) {
        for id in ids {
            self.entities.remove(&id);
            self.local_status_effect_registry.remove(&id);
        }
    }

    pub fn promote_to_player(&mut self, now: DateTime<Utc>, id: u64, skill_id: u32) {
        let entity = match self.entities.get_mut(&id) {
            Some(entity) => entity,
            None => return,
        };

        if let Some(class_id) = SKILL_DATA.get(&skill_id).map(|pr| pr.class_id).filter(|&pr| pr != 0) {
            if let Some(player) = entity.as_player_mut() {
                if player.class_id != class_id {
                    player.class_id = class_id;
                }
            }
            else {
                let entity = Entity::unknown_player(id, class_id, now);
                self.entities.insert(id, entity);
            }
        }
    }

    pub fn update_player(
        &mut self,
        character_id: u64,
        party_instance_id: u32,
        raid_instance_id: u32
    ) {
        if let Some(player) = self.players_by_character_id.get_mut(&character_id) {
            player.party_id = party_instance_id;
        }
    }

    fn get_removed_effects(
        &mut self,
        target_type: StatusEffectTargetType,
        target_id: u64,
        instance_ids: Vec<u32>,
        reason: u8,
    ) -> Vec<StatusEffectDetails> {
        let registry = self.get_registry_mut(target_type, target_id);

        let mut effects = Vec::new();

        if let Some(ser) = registry {
            for id in instance_ids {
                if let Some(status_effect) = ser.remove(&id) {
                    effects.push(status_effect);
                }
            }
        }

        effects
    }

    pub fn on_status_effect_remove(
        &mut self,
        source_id: u64,
        reason: u8,
        instance_ids: Vec<u32>,
        recorded_on: DateTime<Utc>
    ) {
        let is_break = reason == 4;
        let effects = self.get_removed_effects(StatusEffectTargetType::Local, source_id, instance_ids, reason);
        
        if effects.is_empty() {
            if let Some(boss) = self.get_source_entity(source_id, recorded_on).as_boss_mut() {
                boss.encounter_stats.current_shield = 0;
            }

            return;
        }

        for effect in effects {
            let source_id = effect.source_id;
            let target_id = effect.target_id;

            if effect.status_effect_type == StatusEffectType::HardCrowdControl {
                if let Some(player) = self.get_source_entity(target_id, recorded_on).as_player_mut() {
                    player.on_cc_removed(recorded_on, &effect);
                }
            }

            if effect.status_effect_type == StatusEffectType::Shield && is_break {
                let buff_id = effect.status_effect_id;
                let value = effect.value;

                if value == 0 {
                    return;
                }
                
                let source = Self::get_source_entity_unsafe(self, source_id, recorded_on);
                let target = Self::get_source_entity_unsafe(self, target_id, recorded_on);

                match (source.deref_mut(), target.deref_mut()) {
                    (EntityVariant::Player(source), EntityVariant::Player(target)) => {
                        self.damage_stats.total_effective_shielding += value;

                        target.encounter_stats.damage_stats.absorbed += value;
                        target.encounter_stats.damage_stats.absorbed_by.entry(buff_id)
                            .and_modify(|e| *e += value)
                            .or_insert(value);
                        source.encounter_stats.damage_stats.absorbed_on_others += value;
                        source.encounter_stats.damage_stats
                            .absorbed_on_others_by
                            .entry(buff_id)
                            .and_modify(|e| *e += value)
                    .or_insert(value);
                    },
                    (_, EntityVariant::Boss(target)) => {
                        target.encounter_stats.current_shield = value;
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn on_status_effect_add(
        &mut self,
        sed: &StatusEffectData,
        target_id: u64,
        recorded_on: DateTime<Utc>
    ) {
        let source_entity = Self::get_source_entity_unsafe(self, sed.source_id, recorded_on);
        let status_effect = build_status_effect(
            sed.clone(),
            target_id,
            source_entity.id(),
            StatusEffectTargetType::Local,
            recorded_on,
        );

        let target_id = status_effect.target_id;  
        let status_effect_id = status_effect.status_effect_id;
        let value = status_effect.value;

        if status_effect.status_effect_type == StatusEffectType::Shield {
            let target = self.get_source_entity(target_id, recorded_on);
            
            if let Some((source, target)) = source_entity.as_player_mut().zip(target.as_player_mut()) {
                source.on_shield_received(status_effect_id, value);
                target.on_shield_given(status_effect_id, value);
            }
            
            if let Some(target) = target.as_boss_mut() {
                target.current_shield = value;
            }
        }

        if status_effect.status_effect_type == StatusEffectType::HardCrowdControl {
            let target = self.get_source_entity(status_effect.target_id, recorded_on);
            
            if let Some(player) = target.as_player_mut() {
                player.on_cc_applied(&status_effect);                    
            }
        }

        let registry = &mut self.local_status_effect_registry;
        let sub_registry = registry.entry(target_id).or_insert_with(HashMap::new);
        sub_registry.insert(status_effect.instance_id, status_effect);
    }

    pub fn build_and_register_status_effect(
        &mut self,
        sed: &StatusEffectData,
        target_id: u64,
        recorded_on: DateTime<Utc>
    ) {
        let source_entity = self.get_source_entity(sed.source_id, recorded_on);
        let status_effect = build_status_effect(
            sed.clone(),
            target_id,
            source_entity.id(),
            StatusEffectTargetType::Local,
            recorded_on,
        );

        let registry = &mut self.local_status_effect_registry;
        let sub_registry = registry.entry(status_effect.target_id).or_insert_with(HashMap::new);

        sub_registry.insert(status_effect.instance_id, status_effect);
    }

    pub fn get_or_create_entity(&mut self, id: u64, recorded_on: DateTime<Utc>) -> &mut Entity {
        self.entities
            .entry(id)
            .or_insert_with(|| Entity::unknown(id, recorded_on))
    }

    // keep all player entities, reset all stats
    pub fn soft_reset(&mut self, keep_bosses: bool) {

        self.is_resetting = false;
        self.saved = false;
        self.party_freeze = false;

        self.started_on = DateTime::<Utc>::MIN_UTC;
        self.boss_only_damage = self.boss_only_damage;
        self.entities = HashMap::new();
        self.current_boss.id = 0;
        self.current_boss.name = String::from("").into();
        self.damage_stats = Default::default();
        self.raid_clear = false;

        self.damage_log = HashMap::new();
        self.cast_log = HashMap::new();
        self.boss_hp_log = HashMap::new();
        self.party_info = Vec::new();
        self.ntp_fight_start = 0;
        self.rdps_valid = false;

        self.custom_id_map = HashMap::new();

        for (_, entity) in self.entities.iter_mut() {
            match entity.deref_mut() {
                EntityVariant::Player(player) => {
                    player.encounter_stats = Default::default();
                },
                EntityVariant::Boss(boss) => {
                    boss.encounter_stats = Default::default();
                },
                EntityVariant::Esther(esther) => {
                    esther.damage_dealt = Default::default();
                },
                _ => {}
            }
        }
    }

    pub unsafe fn on_skill_start(
        &mut self,
        source_id: u64,
        skill_id: u32,
        tripod_index: Option<TripodIndex>,
        tripod_level: Option<TripodLevel>,
        recorded_on: DateTime<Utc>,
    ) {

        let relative_timestamp = (recorded_on - self.started_on).num_milliseconds();
        let self_ptr = self as *mut Self;

        let player = match self.get_source_entity(source_id, recorded_on).as_player_mut() {
            Some(player) => player,
            None => return,
        };
        
        let mut tripod_change = false;
        let (skill_name, skill_icon, summon_source, skill_type) = unsafe { get_skill(
            &skill_id,
            &(*self_ptr).skill_timestamp,
            player.id,
        ) };

        let skills = &mut player.encounter_stats.skills;
        let skill_stat;
        
        if let Some(stat) = skills.values_mut().find(|s| s.name == skill_name.clone()) {
            skill_stat = stat;
        }
        else {
            skill_stat = player.encounter_stats.skills.entry(skill_id).or_default();
            tripod_change = true;
        }

        tripod_change = check_tripod_index_change(skill_stat.tripod_index, tripod_index)
            || check_tripod_level_change(skill_stat.tripod_level, tripod_level);
        skill_stat.tripod_index = tripod_index;
        skill_stat.tripod_level = tripod_level;
        skill_stat.name = skill_name;
        skill_stat.casts += 1;

        player.encounter_stats.is_dead = false;
        player.encounter_stats.skill_stats.casts += 1;

        if tripod_change {
            if let (Some(tripod_index), Some(_tripod_level)) = (tripod_index, tripod_level) {
                let mut indexes = vec![tripod_index.first];
                if tripod_index.second != 0 {
                    indexes.push(tripod_index.second + 3);
                }
                // third row should never be set if second is not set
                if tripod_index.third != 0 {
                    indexes.push(tripod_index.third + 6);
                }
            }
        }

        &(*self_ptr).cast_log
            .entry(player.id)
            .or_default()
            .entry(skill_id)
            .or_default()
            .push(relative_timestamp as i32);

        if skill_type == "getup" {
           player.shorten_incapacitation(recorded_on);
        }

        let entity_id = player.id;

        for source in summon_source.iter().flatten() {
            let key = (entity_id, *source);
            if self.skill_timestamp.get(&key).is_some() {
                return;
            }
        }

        let key = (entity_id, skill_id);
        self.skill_timestamp.insert(key, relative_timestamp);

        let key = (entity_id, skill_id, relative_timestamp);
        self.skills.insert(
            key,
            SkillCast {
                hits: Vec::new(),
                recorded_on: relative_timestamp,
                last_recorded_on: relative_timestamp,
            },
        );
    }

    pub fn on_damage(
        &mut self,
        damage_data: DamageData,
        source: &mut Entity,
        target: &mut Entity) {

        let DamageData {
            recorded_on,
            mut damage,
            target_current_hp,
            target_max_hp,
            ..
        } = damage_data;

       match (source.deref_mut(), target.deref_mut()) {
            (EntityVariant::Player(player), crate::entity::EntityVariant::Boss(boss)) => {
                self.on_damage_player_to_boss(damage_data, player, boss);
            },
            (EntityVariant::Boss(boss), crate::entity::EntityVariant::Player(player)) => {

                self.damage_stats.top_damage_taken += damage;
                self.damage_stats.top_damage_taken = max(
                self.damage_stats.top_damage_taken,
                player.encounter_stats.damage_taken,
            );
            },
            (EntityVariant::Esther(esther), crate::entity::EntityVariant::Boss(boss)) => {

                if target_current_hp < 0 {
                    damage += target_current_hp;
                }

                esther.damage_dealt += damage;

                boss.encounter_stats.current_hp = target_current_hp;
                boss.encounter_stats.max_hp = target_max_hp;
            },
            _ => {}
        }

        self.updated_on = recorded_on;

    }

    pub fn on_damage_player_to_boss(
        &mut self,
        damage_data: DamageData,
        player: &mut Player,
        boss: &mut Boss) {

        let DamageData {
            is_initial,
            hit_flag,
            hit_option,
            recorded_on,
            mut damage,
            target_current_hp,
            target_max_hp,
            source_id,
            skill_effect_id,
            skill_id,
            ..
        } = damage_data;

        if let Some(skill_id) = skill_id && is_initial {
            let entity_id = player.id;
            let delta_millis = (recorded_on - self.started_on).num_milliseconds();
            self.skill_timestamp.insert((entity_id, skill_id), delta_millis);
            self.skills.insert(
                (entity_id, skill_id, delta_millis),
                SkillCast {
                    hits: Vec::new(),
                    recorded_on: delta_millis,
                    last_recorded_on: delta_millis,
                },
            );
        }

        if target_current_hp < 0 {
            damage += target_current_hp;
        }

        boss.encounter_stats.current_hp = target_current_hp;
        boss.encounter_stats.max_hp = target_max_hp;

        let skill = resolve_skill_or_skill_effect(skill_id, skill_effect_id, &self.skill_timestamp, player.id);
        let relative_timestamp = (recorded_on - self.started_on).num_milliseconds();

        let mut skill_hit = SkillHit {
            damage,
            recorded_on: relative_timestamp,
            ..Default::default()
        };

        let skills = &mut player.encounter_stats.skills;

        let cloned_skill = skill.clone();
        let skill_stat = skills.entry(skill.id).or_insert_with(move || {
            Skill {
                id: cloned_skill.id,
                name: cloned_skill.name,
                icon: cloned_skill.icon,
                summon_sources: Some(cloned_skill.sources),
                casts: 1,
                ..Default::default()
            }
        });

        skill_stat.total_damage += damage;
        if damage > skill_stat.max_damage {
            skill_stat.max_damage = damage;
        }
        skill_stat.updated_on = recorded_on;

        player.encounter_stats.damage_stats.dealt += damage;

            if skill.is_hyper_awakening {
            player.encounter_stats.hyper_awakening_damage += damage;
        }

        player.encounter_stats.skill_stats.hits += 1;
        skill_stat.hits += 1;

        if hit_flag == HitFlag::Critical || hit_flag == HitFlag::DotCritical {
            player.encounter_stats.skill_stats.crits += 1;
            player.encounter_stats.damage_stats.crit += damage;
            skill_stat.crits += 1;
            skill_stat.crit_damage += damage;
            skill_hit.crit = true;
        }

        if hit_option == HitOption::BackAttack {
            player.encounter_stats.skill_stats.back_attacks += 1;
            player.encounter_stats.damage_stats.back_attack += damage;
            skill_stat.back_attacks += 1;
            skill_stat.back_attack_damage += damage;
            skill_hit.back_attack = true;
        }

        if hit_option == HitOption::FrontalAttack {
            player.encounter_stats.skill_stats.front_attacks += 1;
            player.encounter_stats.damage_stats.front_attack += damage;
            skill_stat.front_attacks += 1;
            skill_stat.front_attack_damage += damage;
            skill_hit.front_attack = true;
        }

        self.damage_stats.total_damage_dealt += damage;
        self.damage_stats.top_damage_dealt = max(
        self.damage_stats.top_damage_dealt,
        player.encounter_stats.damage_stats.dealt,
        );  

        self.damage_log
            .entry(player.id)
            .or_default()
            .push((relative_timestamp, damage));

        let mut is_buffed_by_support = false;
        let mut is_buffed_by_identity = false;
        let mut is_debuffed_by_support = false;
        let mut is_buffed_by_hat = false;
        let self_ptr = self as *mut Self;

        let (se_on_source, se_on_target) = unsafe {self.get_status_effects_player_vs_boss(
            recorded_on,
            player.id,
            player.character_id,
            boss.id)
        };

        let se_on_source_ids = se_on_source
            .iter()
            .map(|se| map_status_effect(se))
            .collect::<Vec<_>>();

        let damage_stats = unsafe { &mut (*self_ptr).damage_stats };
        
        for buff_id in se_on_source_ids.iter() {    
            let is_hat = is_hat_buff(buff_id);

            let is_not_tracked = !damage_stats.unknown_buffs.contains(buff_id)
                && !damage_stats.buffs.contains_key(buff_id);

            if is_not_tracked {
                let (original_buff_id, source_id) = unsafe { match (*self_ptr).custom_id_map.get(buff_id) {
                    Some(&deref_id) => (deref_id, Some(get_skill_id(*buff_id, deref_id))),
                    None => (*buff_id, None),
                } };
                
                if let Some(status_effect) = get_status_effect_data(original_buff_id, source_id) {
                    damage_stats.buffs.insert(*buff_id, status_effect);
                } else {
                    damage_stats.unknown_buffs.insert(*buff_id);
                }   
            }

            let buff = damage_stats.buffs.get(buff_id);
            let combined = (damage_stats.buffs.get(buff_id)
                .and_then(|pr| pr.source.skill.as_ref()
                    .map(|sk| (is_support_class_id(sk.class_id), pr.buff_type, pr.target, pr.buff_category.as_str()))));

            if let Some((is_support, buff_type, target, buff_category)) = 
                combined && !is_buffed_by_support && !is_hat {
                is_buffed_by_support = is_support
                    && buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                    && target == StatusEffectTarget::PARTY
                    && (buff_category == "classskill"
                        || buff_category == "arkpassive");
            }

            if let Some((is_support, buff_type, target, buff_category)) = 
                combined && !is_buffed_by_identity {
                is_buffed_by_identity = is_support
                    && buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                    && target == StatusEffectTarget::PARTY
                    && buff_category == "identity";
            }

            if !is_buffed_by_hat && is_hat {
                is_buffed_by_hat = true;
            }
        }

        let se_on_target_ids = se_on_target
            .iter()
            .map(|se| map_status_effect(se))
            .collect::<Vec<_>>();

        for buff_id in se_on_target_ids.iter() {
            let is_hat = is_hat_buff(buff_id);

            let is_not_tracked = !damage_stats.unknown_buffs.contains(buff_id)
                && !damage_stats.debuffs.contains_key(buff_id);

            if is_not_tracked {
                let (original_buff_id, source_id) = unsafe { match (*self_ptr).custom_id_map.get(buff_id) {
                    Some(&deref_id) => (deref_id, Some(get_skill_id(*buff_id, deref_id))),
                    None => (*buff_id, None),
                } };
                
                if let Some(status_effect) = get_status_effect_data(original_buff_id, source_id) {
                    damage_stats.debuffs.insert(*buff_id, status_effect);
                } else {
                    damage_stats.unknown_buffs.insert(*buff_id);
                }   
            }

            let combined = (damage_stats.debuffs.get(buff_id)
                .and_then(|pr| pr.source.skill.as_ref()
                    .map(|sk| (is_support_class_id(sk.class_id), pr.buff_type, pr.target))));

            if let Some((is_support, buff_type, target)) = 
                combined && !is_debuffed_by_support {
                is_debuffed_by_support = is_support
                    && buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                    && target == StatusEffectTarget::PARTY;
            }
        }

        let is_hyper_awakening = skill.is_hyper_awakening;

        if is_buffed_by_support && !is_hyper_awakening {
            skill_stat.buffed_by_support += damage;
            player.encounter_stats.buffed_by_support += damage;
        }
        if is_buffed_by_identity && !is_hyper_awakening {
            skill_stat.buffed_by_identity += damage;
            player.encounter_stats.buffed_by_identity += damage;
        }
        if is_debuffed_by_support && !is_hyper_awakening {
            skill_stat.debuffed_by_support += damage;
            player.encounter_stats.debuffed_by_support += damage;
        }
        if is_buffed_by_hat {
            skill_stat.buffed_by_hat += damage;
            player.encounter_stats.buffed_by_hat += damage;
        }

        let stabilized_status_active =
            (player.encounter_stats.current_hp as f64 / player.encounter_stats.max_hp as f64) > 0.65;
        let mut filtered_se_on_source_ids: Vec<u32> = vec![];

        for buff_id in se_on_source_ids.iter() {
            if is_hyper_awakening && !is_hat_buff(buff_id) {
                continue;
            }

            if let Some(buff) = self.damage_stats.buffs.get(buff_id) {
                if !stabilized_status_active && buff.source.name.contains("Stabilized Status") {
                    continue;
                }
            }

            filtered_se_on_source_ids.push(*buff_id);

            skill_stat.buffed_by
                .entry(*buff_id)
                .and_modify(|e| *e += damage)
                .or_insert(damage);
            player.encounter_stats
                .buffed_by
                .entry(*buff_id)
                .and_modify(|e| *e += damage)
                .or_insert(damage);
        }

        for debuff_id in se_on_target_ids.iter() {
            if is_hyper_awakening {
                break;
            }

            skill_stat.debuffed_by
                .entry(*debuff_id)
                .and_modify(|e| *e += damage)
                .or_insert(damage);
            player.encounter_stats
                .debuffed_by
                .entry(*debuff_id)
                .and_modify(|e| *e += damage)
                .or_insert(damage);
        }

        skill_hit.buffed_by = filtered_se_on_source_ids;
        if !is_hyper_awakening {
            skill_hit.debuffed_by = se_on_target_ids;
        }
        
        if !self.current_boss.name.eq(&boss.name) {
            self.current_boss.id = boss.id;
            self.current_boss.name = boss.name.clone();
        }

        let log = self
            .boss_hp_log
            .entry(boss.name.to_string())
            .or_default();

        let current_hp = if boss.encounter_stats.current_hp >= 0 {
            boss.encounter_stats.current_hp + boss.encounter_stats.current_shield as i64
        } else {
            0
        };
        let hp_percent = if boss.encounter_stats.max_hp != 0 {
            current_hp as f32 / boss.encounter_stats.max_hp as f32
        } else {
            0.0
        };

        let relative_timestamp_s = relative_timestamp as i32 / 1000;

        if log.is_empty() || log.last().unwrap().time != relative_timestamp_s {
            log.push(BossHpLog::new(relative_timestamp_s, current_hp, hp_percent));
        } else {
            let last = log.last_mut().unwrap();
            last.hp = current_hp;
            last.p = hp_percent;
        }

        if skill.id > 0 {
            self.on_hit(
                player.id,
                source_id,
                skill.id,
                &skill_hit,
                &skill.sources,
            );
        }
    }

    pub fn get_player_by_id(&mut self, id: u64) -> Option<&mut Player> {
        self.entities.get_mut(&id).and_then(|pr| pr.as_player_mut())
    }

}