use anyhow::*;
use flate2::read::GzDecoder;
use hashbrown::HashMap;
use serde_json::json;
use std::{fs, io::Read, path::PathBuf, str::FromStr};

use log::*;
use rusqlite::{params, params_from_iter, Connection, Row, Transaction};

use crate::{database::{queries::*}, models::*, misc::utils::compress_json};


pub fn parse_encounter(row: &Row) -> rusqlite::Result<(Encounter, bool)> {
    
    let mut compressed = false;
    let misc_str: String = row.get(12).unwrap_or_default();
    let misc = serde_json::from_str::<EncounterMisc>(misc_str.as_str())
        .map(Some)
        .unwrap_or_default();

    let mut boss_hp_log: HashMap<String, Vec<BossHpLog>> = HashMap::new();

    if let Some(misc) = misc.as_ref() {
        let version = misc
            .version
            .clone()
            .unwrap_or_default()
            .split('.')
            .map(|x| x.parse::<i32>().unwrap_or_default())
            .collect::<Vec<_>>();

        if version[0] > 1
            || (version[0] == 1 && version[1] >= 14)
            || (version[0] == 1 && version[1] == 13 && version[2] >= 5)
        {
            compressed = true;
        }

        if !compressed {
            boss_hp_log = misc.boss_hp_log.clone().unwrap_or_default();
        }
    }

    let buffs: HashMap<u32, StatusEffect>;
    let debuffs: HashMap<u32, StatusEffect>;
    let applied_shield_buffs: HashMap<u32, StatusEffect>;

    if compressed {
        let raw_bytes: Vec<u8> = row.get(10).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut buff_string = String::new();
        decompress.read_to_string(&mut buff_string).unwrap();
        buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_string.as_str()).unwrap_or_default();

        let raw_bytes: Vec<u8> = row.get(11).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut debuff_bytes = vec![];
        // decompress.read_to_string(&mut debuff_string)?;
        decompress.read_to_end(&mut debuff_bytes).unwrap();
        debuffs = serde_json::from_slice::<HashMap<u32, StatusEffect>>(&debuff_bytes).unwrap_or_default();

        let raw_bytes: Vec<u8> = row.get(19).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut applied_shield_buff_string = String::new();
        decompress
            .read_to_string(&mut applied_shield_buff_string)
            .expect("could not decompress applied_shield_buffs");
        applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
            applied_shield_buff_string.as_str(),
        )
        .unwrap_or_default();

        let raw_bytes: Vec<u8> = row.get(20).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut boss_string = String::new();
        decompress
            .read_to_string(&mut boss_string)
            .expect("could not decompress boss_hp_log");
        boss_hp_log =
            serde_json::from_str::<HashMap<String, Vec<BossHpLog>>>(boss_string.as_str())
                .unwrap_or_default();
    } else {
        let buff_str: String = row.get(10).unwrap_or_default();
        buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_str.as_str())
            .unwrap_or_default();
        let debuff_str: String = row.get(11).unwrap_or_default();
        debuffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(debuff_str.as_str())
            .unwrap_or_default();
        let applied_shield_buff_str: String = row.get(19).unwrap_or_default();
        applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
            applied_shield_buff_str.as_str(),
        )
        .unwrap_or_default();
    }

    let total_shielding = row.get(17).unwrap_or_default();
    let total_effective_shielding = row.get(18).unwrap_or_default();

    let encounter = Encounter {
        last_combat_packet: row.get(0)?,
        fight_start: row.get(1)?,
        local_player: row.get(2).unwrap_or("You".to_string()),
        current_boss_name: row.get(3)?,
        duration: row.get(4)?,
        encounter_damage_stats: EncounterDamageStats {
            total_damage_dealt: row.get(5)?,
            top_damage_dealt: row.get(6)?,
            total_damage_taken: row.get(7)?,
            top_damage_taken: row.get(8)?,
            dps: row.get(9)?,
            buffs,
            debuffs,
            misc,
            total_shielding,
            total_effective_shielding,
            applied_shield_buffs,
            boss_hp_log,
            ..Default::default()
        },
        difficulty: row.get(13)?,
        favorite: row.get(14)?,
        cleared: row.get(15)?,
        boss_only_damage: row.get(16)?,
        ..Default::default()
    };

    rusqlite::Result::Ok((encounter, compressed))
}

pub fn parse_entity(row: &Row, compressed: bool) -> rusqlite::Result<EncounterEntity> {
    
    let skills: HashMap<u32, Skill>;
    let damage_stats: DamageStats;

    if compressed {
        let raw_bytes: Vec<u8> = row.get(7).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut skill_string = String::new();
        decompress
            .read_to_string(&mut skill_string)
            .expect("could not decompress skills");
        skills = serde_json::from_str::<HashMap<u32, Skill>>(skill_string.as_str())
            .unwrap_or_default();

        let raw_bytes: Vec<u8> = row.get(8).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut damage_stats_string = String::new();
        decompress
            .read_to_string(&mut damage_stats_string)
            .expect("could not decompress damage stats");
        damage_stats = serde_json::from_str::<DamageStats>(damage_stats_string.as_str())
            .unwrap_or_default();
    } else {
        let skill_str: String = row.get(7).unwrap_or_default();
        skills = serde_json::from_str::<HashMap<u32, Skill>>(skill_str.as_str())
            .unwrap_or_default();

        let damage_stats_str: String = row.get(8).unwrap_or_default();
        damage_stats = serde_json::from_str::<DamageStats>(damage_stats_str.as_str())
            .unwrap_or_default();
    }

    let skill_stats_str: String = row.get(9).unwrap_or_default();
    let skill_stats =
        serde_json::from_str::<SkillStats>(skill_stats_str.as_str()).unwrap_or_default();

    let entity_type: String = row.get(11).unwrap_or_default();

    let engravings_str: String = row.get(14).unwrap_or_default();
    let engravings = serde_json::from_str::<Option<Vec<String>>>(engravings_str.as_str())
        .unwrap_or_default();

    let spec: Option<String> = row.get(15).unwrap_or_default();
    let ark_passive_active: Option<bool> = row.get(16).unwrap_or_default();

    let ark_passive_data_str: String = row.get(17).unwrap_or_default();
    let ark_passive_data =
        serde_json::from_str::<Option<ArkPassiveData>>(ark_passive_data_str.as_str())
            .unwrap_or_default();

    let entity = EncounterEntity {
        name: row.get(0)?,
        class_id: row.get(1)?,
        class: row.get(2)?,
        gear_score: row.get(3)?,
        current_hp: row.get(4)?,
        max_hp: row.get(5)?,
        is_dead: row.get(6)?,
        skills,
        damage_stats,
        skill_stats,
        entity_type: EntityType::from_str(entity_type.as_str())
            .unwrap_or(EntityType::Unknown),
        npc_id: row.get(12)?,
        character_id: row.get(13).unwrap_or_default(),
        engraving_data: engravings,
        spec,
        ark_passive_active,
        ark_passive_data,
        ..Default::default()
    };

    rusqlite::Result::Ok(entity)
}

pub fn parse_encounter_preview(row: &Row) -> rusqlite::Result<EncounterPreview> {
    let classes: String = row.get(9).unwrap_or_default();

    let (classes, names) = classes
        .split(',')
        .map(|s| {
            let info: Vec<&str> = s.split(':').collect();
            if info.len() != 2 {
                return (101, "Unknown".to_string());
            }
            (info[0].parse::<i32>().unwrap_or(101), info[1].to_string())
        })
        .unzip();

    let result = EncounterPreview {
        id: row.get(0)?,
        fight_start: row.get(1)?,
        boss_name: row.get(2)?,
        duration: row.get(3)?,
        classes,
        names,
        difficulty: row.get(4)?,
        favorite: row.get(5)?,
        cleared: row.get(6)?,
        local_player: row.get(7)?,
        my_dps: row.get(8).unwrap_or(0),
    };

    rusqlite::Result::Ok(result)
}