use crate::constants::{WINDOW_MS, WINDOW_S};
use crate::core::stats_api::{PlayerStats, StatsApi};
use crate::database::SaveToDb;
use crate::{constants::TIMEOUT_DELAY_MS, database::Database};
use crate::models::*;
use crate::misc::data::*;
use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use log::{error, warn};
use meter_core::packets::structures::{StatPair, StatusEffectData};
use moka::sync::Cache;
use tauri::{AppHandle, Emitter, EventTarget};
use tokio::task;
use std::collections::BTreeMap;
use std::{cmp::{max, Ordering, Reverse}, sync::Arc};

pub async fn get_player_info(stats_api: Arc<StatsApi>, model: &SaveToDb) -> Option<HashMap<String, PlayerStats>> {
    if model.raid_difficulty == RaidDifficulty::Unknown 
        || model.current_boss_name.is_empty() {
        return None
    }

    let region = match model.region.clone() {
        Some(region) => region,
        None => {
            warn!("region is not set");
            return None;
        }
    };

    let raid_name = boss_to_raid_map(&model.current_boss_name, model.boss_max_hp).unwrap_or_default();

    let player_names: Vec<String> = model.entities.iter()
        .filter_map(|e| {
            if is_valid_player(e) {
                Some(e.name.clone())
            } else {
                None
            }
        })
        .collect();

    if player_names.len() > 16 {
        return None;
    }

    let info = stats_api.get_character_info(
        &model.version,
        region,
        raid_name,
        model.raid_difficulty.as_ref(),
        model.raid_clear,
        &model.current_boss_name,
        player_names).await;

    info
}

pub fn save_to_db(
    app_handle: AppHandle,
    stats_api: Arc<StatsApi>,
    database: Arc<Database>,
    mut model: SaveToDb) {
    let app_handle = app_handle.clone();
    
    task::spawn(async move {

        let player_info = get_player_info(stats_api, &model).await;

        calculate_stats(
            &mut model.entities,
            model.started_on.timestamp_millis(),
            model.updated_on.timestamp_millis(),
            model.duration_seconds,
            &model.cast_log,
            &model.skill_cast_log,
            player_info,
            &model.encounter_damage_stats,
            &model.damage_log);

        match database.insert_data(model) {
            Ok(encounter_id) => {
                app_handle.emit_to(EventTarget::Any, "clear-encounter", encounter_id).unwrap();
            },
            Err(err) => error!("An error occurred whilst saving to database: {}", err),
        };
    });
}

pub fn calculate_stats(
    entities: &mut Vec<EncounterEntity>,
    fight_start: i64,
    fight_end: i64,
    duration_seconds: i64,
    cast_log: &HashMap<u64, HashMap<u32, Vec<i32>>>,
    skill_cast_log: &HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
    player_info: Option<HashMap<String, PlayerStats>>,
    encounter_damage_stats: &EncounterDamageStats,
    damage_log: &HashMap<u64, Vec<(i64, i64)>>
) -> anyhow::Result<()> {
    let fight_start_sec = fight_start / 1000;
    let fight_end_sec = fight_end / 1000;

    for entity in entities {
        if entity.entity_type == EntityType::Player {
            let intervals = generate_intervals(fight_start, fight_end);
            if let Some(damage_log) = damage_log.get(&entity.id) {
                if !intervals.is_empty() {
                    for interval in intervals {
                        let start = fight_start + interval - WINDOW_MS;
                        let end = fight_start + interval + WINDOW_MS;

                        let damage = sum_in_range(damage_log, start, end);
                        entity
                            .damage_stats
                            .dps_rolling_10s_avg
                            .push(damage / (WINDOW_S * 2));
                    }
                }
                
                entity.damage_stats.dps_average =
                    calculate_average_dps(damage_log, fight_start_sec, fight_end_sec);
            }

            let spec = get_player_spec(entity, &encounter_damage_stats.buffs);

            entity.spec = Some(spec.clone());

            if let Some(info) = player_info
                .as_ref()
                .and_then(|stats| stats.get(&entity.name))
            {
                for gem in info.gems.iter().flatten() {
                    for skill_id in gem_skill_id_to_skill_ids(gem.skill_id) {
                        if let Some(skill) = entity.skills.get_mut(&skill_id) {
                            match gem.gem_type {
                                5 | 34 => {
                                    // damage gem
                                    skill.gem_damage =
                                        Some(damage_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                                27 | 35 => {
                                    // cooldown gem
                                    skill.gem_cooldown =
                                        Some(cooldown_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier = Some(gem.tier);
                                }
                                64 | 65 => {
                                    // support identity gem??
                                    skill.gem_damage =
                                        Some(support_damage_gem_value_to_level(gem.value));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                entity.ark_passive_active = Some(info.ark_passive_enabled);

                let engravings = get_engravings(&info.engravings);
                if entity.class_id == 104
                    && engravings.as_ref().is_some_and(|engravings| {
                        engravings
                            .iter()
                            .any(|e| e == "Awakening" || e == "Drops of Ether")
                    })
                {
                    entity.spec = Some("Princess".to_string());
                } else if spec == "Unknown" {
                    // not reliable enough to be used on its own
                    if let Some(tree) = info.ark_passive_data.as_ref() {
                        if let Some(enlightenment) = tree.enlightenment.as_ref() {
                            for node in enlightenment.iter() {
                                let spec = get_spec_from_ark_passive(node);
                                if spec != "Unknown" {
                                    entity.spec = Some(spec);
                                    break;
                                }
                            }
                        }
                    }
                }

                entity.engraving_data = engravings;
                entity.ark_passive_data = info.ark_passive_data.clone();
            }
        }

        entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;

        for (_, skill) in entity.skills.iter_mut() {
            skill.dps = skill.total_damage / duration_seconds;
        }

        for (_, cast_log) in cast_log.iter().filter(|&(s, _)| *s == entity.id) {
            for (skill, log) in cast_log {
                entity.skills.entry(*skill).and_modify(|e| {
                    e.cast_log.clone_from(log);
                });
            }
        }

        for (_, skill_cast_log) in skill_cast_log.iter().filter(|&(s, _)| *s == entity.id) {
            for (skill, log) in skill_cast_log {
                entity.skills.entry(*skill).and_modify(|e| {
                    let average_cast = e.total_damage as f64 / e.casts as f64;
                    let filter = average_cast * 0.05;
                    let mut adj_hits = 0;
                    let mut adj_crits = 0;
                    for cast in log.values() {
                        for hit in cast.hits.iter() {
                            if hit.damage as f64 > filter {
                                adj_hits += 1;
                                if hit.crit {
                                    adj_crits += 1;
                                }
                            }
                        }
                    }

                    if adj_hits > 0 {
                        e.adjusted_crit = Some(adj_crits as f64 / adj_hits as f64);
                    }

                    e.max_damage_cast = log
                        .values()
                        .map(|cast| cast.hits.iter().map(|hit| hit.damage).sum::<i64>())
                        .max()
                        .unwrap_or_default();
                    e.skill_cast_log = log.values().cloned().collect();
                });
            }
        }
    }

    Ok(())
}

pub fn is_support_class_id(class_id: u32) -> bool {
    class_id == 105 || class_id == 204 || class_id == 602
}

pub fn is_battle_item(skill_effect_id: &u32, _item_type: &str) -> bool {
    SKILL_EFFECT_DATA
        .get(skill_effect_id).iter()
        .filter_map(|&pr| pr.item_type.as_ref().filter(|np| *np == "useup"))
        .next().is_some()
}

pub fn get_status_effect_data(buff_id: u32, source_skill: Option<u32>) -> Option<StatusEffect> {
    let buff = SKILL_BUFF_DATA.get(&buff_id)?;

    if buff.icon_show_type.as_deref().unwrap_or_default() == "none" {
        return None;
    }

    let raw_buff_category = buff.buff_category.as_deref().unwrap_or_default();
    let buff_category = if raw_buff_category == "ability"
        && matches!(buff.unique_group, 501 | 502 | 503 | 504 | 505)
    {
        "dropsofether"
    } else {
        raw_buff_category
    };

    let target = match buff.target.as_str() {
        "none" => StatusEffectTarget::OTHER,
        "self" => StatusEffectTarget::SELF,
        _ => StatusEffectTarget::PARTY,
    };

    let mut status_effect = StatusEffect {
        target,
        category: buff.category.clone(),
        buff_category: buff_category.to_string(),
        buff_type: get_status_effect_buff_type_flags(buff),
        unique_group: buff.unique_group,
        source: StatusEffectSource {
            name: buff.name.clone()?,
            desc: buff.desc.clone()?,
            icon: buff.icon.clone()?,
            ..Default::default()
        },
    };

    match buff_category {
        "classskill" | "arkpassive" | "identity"
            | "ability" if buff.unique_group != 0 => {
            if let Some(source_skills) = buff.source_skills.as_ref() {
                let skill_id = source_skill.or_else(|| source_skills.first().copied()).unwrap_or(0);
                let skill= SKILL_DATA.get(&skill_id);
                status_effect.source.skill = get_summon_source_skill(skill);
            } else {
                let fallback_ids: [u32; 3] = [
                    buff_id / 10,
                    (buff_id / 100) * 10,
                    buff.unique_group / 10,
                ];
                for id in fallback_ids {
                    if let Some(skill) = SKILL_DATA.get(&id) {
                        status_effect.source.skill = Some(skill.clone());
                        break;
                    }
                }
            }
        }

        "set" => {
            if let Some(name) = &buff.set_name {
                status_effect.source.set_name = Some(name.clone());
            }
        }

        "battleitem" => {
            if let Some(item) = SKILL_EFFECT_DATA.get(&buff_id) {
                if let Some(name) = &item.item_name {
                    status_effect.source.name = name.clone();
                }
                if let Some(desc) = &item.item_desc {
                    status_effect.source.desc = desc.clone();
                }
                if let Some(icon) = &item.icon {
                    status_effect.source.icon = icon.clone();
                }
            }
        }

        _ => {}
    }

    Some(status_effect)
}

fn get_summon_source_skill(skill: Option<&SkillData>) -> Option<SkillData> {
    skill.map(|skill| {
        if let Some(first) = skill.summon_source_skills.as_ref().and_then(|s| s.first()) {
            if *first > 0 {
                if let Some(summon_skill) = SKILL_DATA.get(first) {
                    return summon_skill.clone();
                }
            }
        }
        skill.clone()
    })
}

pub fn get_status_effect_buff_type_flags(buff: &SkillBuffData) -> u32 {
    let dmg_buffs = [
        "weaken_defense",
        "weaken_resistance",
        "skill_damage_amplify",
        "beattacked_damage_amplify",
        "skill_damage_amplify_attack",
        "directional_attack_amplify",
        "instant_stat_amplify",
        "attack_power_amplify",
        "instant_stat_amplify_by_contents",
        "evolution_type_damage",
    ];

    let mut buff_type = StatusEffectBuffTypeFlags::NONE;
    if dmg_buffs.contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::DMG;
    } else if ["move_speed_down", "all_speed_down"].contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::MOVESPEED;
    } else if buff.buff_type == "reset_cooldown" {
        buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
    } else if ["change_ai_point", "ai_point_amplify"].contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::STAGGER;
    } else if buff.buff_type == "increase_identity_gauge" {
        buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
    }

    for option in buff.passive_options.iter() {
        let key_stat_str = option.key_stat.as_str();
        let option_type = option.option_type.as_str();
        if option_type == "stat" {
            let stat = STAT_TYPE_MAP.get(key_stat_str);
            if stat.is_none() {
                continue;
            }
            let stat = stat.unwrap().to_owned();
            if ["mastery", "mastery_x", "paralyzation_point_rate"].contains(&key_stat_str) {
                buff_type |= StatusEffectBuffTypeFlags::STAGGER;
            } else if ["rapidity", "rapidity_x", "cooldown_reduction"].contains(&key_stat_str) {
                buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
            } else if [
                "max_mp",
                "max_mp_x",
                "max_mp_x_x",
                "normal_mp_recovery",
                "combat_mp_recovery",
                "normal_mp_recovery_rate",
                "combat_mp_recovery_rate",
                "resource_recovery_rate",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
            } else if [
                "con",
                "con_x",
                "max_hp",
                "max_hp_x",
                "max_hp_x_x",
                "normal_hp_recovery",
                "combat_hp_recovery",
                "normal_hp_recovery_rate",
                "combat_hp_recovery_rate",
                "self_recovery_rate",
                "drain_hp_dam_rate",
                "vitality",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::HP;
            } else if STAT_TYPE_MAP["def"] <= stat && stat <= STAT_TYPE_MAP["magical_inc_rate"]
                || ["endurance", "endurance_x"].contains(&option.key_stat.as_str())
            {
                if buff.category == "buff" && option.value >= 0
                    || buff.category == "debuff" && option.value <= 0
                {
                    buff_type |= StatusEffectBuffTypeFlags::DMG;
                } else {
                    buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
                }
            } else if STAT_TYPE_MAP["move_speed"] <= stat
                && stat <= STAT_TYPE_MAP["vehicle_move_speed_rate"]
            {
                buff_type |= StatusEffectBuffTypeFlags::MOVESPEED;
            }
            if [
                "attack_speed",
                "attack_speed_rate",
                "rapidity",
                "rapidity_x",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::ATKSPEED;
            } else if ["critical_hit_rate", "criticalhit", "criticalhit_x"].contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::CRIT;
            } else if STAT_TYPE_MAP["attack_power_sub_rate_1"] <= stat
                && stat <= STAT_TYPE_MAP["skill_damage_sub_rate_2"]
                || STAT_TYPE_MAP["fire_dam_rate"] <= stat
                    && stat <= STAT_TYPE_MAP["elements_dam_rate"]
                || [
                    "str",
                    "agi",
                    "int",
                    "str_x",
                    "agi_x",
                    "int_x",
                    "char_attack_dam",
                    "attack_power_rate",
                    "skill_damage_rate",
                    "attack_power_rate_x",
                    "skill_damage_rate_x",
                    "hit_rate",
                    "dodge_rate",
                    "critical_dam_rate",
                    "awakening_dam_rate",
                    "attack_power_addend",
                    "weapon_dam",
                ]
                .contains(&key_stat_str)
            {
                if buff.category == "buff" && option.value >= 0
                    || buff.category == "debuff" && option.value <= 0
                {
                    buff_type |= StatusEffectBuffTypeFlags::DMG;
                } else {
                    buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
                }
            }
        } else if option_type == "skill_critical_ratio" {
            buff_type |= StatusEffectBuffTypeFlags::CRIT;
        } else if [
            "skill_damage",
            "class_option",
            "skill_group_damage",
            "skill_critical_damage",
            "skill_penetration",
        ]
        .contains(&option_type)
        {
            if buff.category == "buff" && option.value >= 0
                || buff.category == "debuff" && option.value <= 0
            {
                buff_type |= StatusEffectBuffTypeFlags::DMG;
            } else {
                buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
            }
        } else if ["skill_cooldown_reduction", "skill_group_cooldown_reduction"]
            .contains(&option_type)
        {
            buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
        } else if ["skill_mana_reduction", "mana_reduction"].contains(&option_type) {
            buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
        } else if option_type == "combat_effect" {
            if let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&option.key_index) {
                for effect in combat_effect.effects.iter() {
                    for action in effect.actions.iter() {
                        if [
                            "modify_damage",
                            "modify_final_damage",
                            "modify_critical_multiplier",
                            "modify_penetration",
                            "modify_penetration_when_critical",
                            "modify_penetration_addend",
                            "modify_penetration_addend_when_critical",
                            "modify_damage_shield_multiplier",
                        ]
                        .contains(&action.action_type.as_str())
                        {
                            buff_type |= StatusEffectBuffTypeFlags::DMG;
                        } else if action.action_type == "modify_critical_ratio" {
                            buff_type |= StatusEffectBuffTypeFlags::CRIT;
                        }
                    }
                }
            }
        }
    }

    buff_type.bits()
}

pub fn get_skill(
    skill_id: &u32,
    skill_timestamp: &Cache<(u64, u32), i64>,
    entity_id: u64,
) -> (String, String, Option<Vec<u32>>, String) {
    let mut skill_name = skill_id.to_string();

   if let Some(skill) = SKILL_DATA.get(skill_id) {
        skill_name = skill.name.clone().unwrap_or_else(|| skill_name);

        if let Some(summon_source_skill) = skill.summon_source_skills.as_ref() {
            for source in summon_source_skill {
                if skill_timestamp.get(&(entity_id, *source)).is_some()
                {
                    if let Some(skill) = SKILL_DATA.get(source) {
                        return (
                            skill.name.clone().unwrap_or_default() + " (Summon)",
                            skill.icon.clone().unwrap_or_default(),
                            Some(summon_source_skill.clone()),
                            skill.skill_type.to_string()
                        );
                    }
                }
            }
            if let Some(skill) = SKILL_DATA.get(summon_source_skill.iter().min().unwrap_or(&0))
            {
                (
                    skill.name.clone().unwrap_or_default() + " (Summon)",
                    skill.icon.clone().unwrap_or_default(),
                    Some(summon_source_skill.clone()),
                    skill.skill_type.to_string()
                )
            } else {
                (skill_name, "".to_string(), None, skill.skill_type.to_string())
            }
        } else if let Some(source_skill) = skill.source_skills.as_ref() {
            if let Some(skill) = SKILL_DATA.get(source_skill.iter().min().unwrap_or(&0)) {
                (
                    skill.name.clone().unwrap_or_default(),
                    skill.icon.clone().unwrap_or_default(),
                    None,
                    skill.skill_type.to_string()
                )
            } else {
                (skill_name, "".to_string(), None, skill.skill_type.to_string())
            }
        } else {
            (
                skill.name.clone().unwrap_or_default(),
                skill.icon.clone().unwrap_or_default(),
                None,
                skill.skill_type.to_string()
            )
        }
    } else if let Some(skill) = SKILL_DATA.get(&(skill_id - (skill_id % 10))) {
        (
            skill.name.clone().unwrap_or_default(),
            skill.icon.clone().unwrap_or_default(),
            None,
            skill.skill_type.to_string()
        )
    } else {
        (skill_name, "".to_string(), None, "".to_string())
    }
}


pub fn resolve_skill_or_skill_effect(
    skill_id: Option<u32>,
    skill_effect_id: Option<u32>,
    skill_timestamp: &Cache<(u64, u32), i64>,
    entity_id: u64,
) -> SkillSlim {
    match (skill_id, skill_effect_id) {
        (None, None) => SkillSlim { id: 0, parent_id: None, name: "Bleed".to_string(), icon: "buff_168.png".to_string(), ..Default::default() },
        (None, Some(skill_effect_id)) => {
            let default_skill_name = skill_effect_id.to_string();
            let default_icon = "".to_string();

            if let Some(effect) = SKILL_EFFECT_DATA.get(&skill_effect_id) {

                if let Some(item_name) = effect.item_name.as_ref() {
                    return SkillSlim { 
                        id: skill_effect_id,
                        parent_id: None,
                        name: item_name.to_string(),
                        icon: effect.icon.clone().unwrap_or_default(),
                        ..Default::default()
                    };
                }

                if let Some(source_skill) = effect.source_skills.as_ref()
                    && let Some(min_id) = source_skill.iter().min()
                    && let Some(skill) = SKILL_DATA.get(min_id) 
                {
                    return SkillSlim {
                        id: skill.id as u32,
                        parent_id: Some(skill_effect_id),
                        is_hyper_awakening: skill.is_hyper_awakening,
                        name: skill.name.clone().unwrap_or_default(),
                        icon: skill.icon.clone().unwrap_or_default(),
                        ..Default::default()
                    };
                }

                let relative_skill_id = skill_effect_id / 10;
                if let Some(skill) = SKILL_DATA.get(&relative_skill_id) {
                    return SkillSlim {
                        id: relative_skill_id,
                        parent_id: Some(skill_effect_id),
                        name: skill.name.clone().unwrap_or_default(),
                        icon: skill.icon.clone().unwrap_or_default(),
                        ..Default::default()
                    };
                }

                return SkillSlim { 
                    id: skill_effect_id,
                    parent_id: None,
                    name: effect.comment.clone(),
                    icon: default_icon,
                    ..Default::default()
                }
            }

            SkillSlim { id: skill_effect_id, parent_id: None, name: default_skill_name, icon: default_icon, ..Default::default() }
        },
        (Some(skill_id), None) => {
            let default_skill_name = skill_id.to_string();
            let default_icon = "".to_string();

            if let Some(skill) = SKILL_DATA.get(&skill_id) {

                if let Some(source_skill) = skill.summon_source_skills.iter().flatten().find_map(|source| {
                    if skill_timestamp.contains_key(&(entity_id, *source)) {
                        SKILL_DATA.get(source)
                    } else {
                        None
                    }
                }) {
                    return SkillSlim { 
                        id: source_skill.id as u32,
                        parent_id: Some(skill_id),
                        is_hyper_awakening: source_skill.is_hyper_awakening,
                        name: source_skill.name.clone().unwrap_or_default() + " (Summon)",
                        icon: source_skill.icon.clone().unwrap_or_default(),
                        sources: skill.summon_source_skills.clone().unwrap_or_default(),
                        ..Default::default()
                    }
                }

                if let Some(source_skill) = skill.summon_source_skills.iter().flatten().min()
                    .and_then(|min_id| SKILL_DATA.get(min_id))
                {
                    return SkillSlim { 
                        id: source_skill.id as u32,
                        parent_id: Some(skill_id),
                        is_hyper_awakening: source_skill.is_hyper_awakening,
                        name: source_skill.name.clone().unwrap_or_default() + " (Summon)",
                        icon: source_skill.icon.clone().unwrap_or_default(),
                        sources: skill.summon_source_skills.clone().unwrap_or_default(),
                        ..Default::default()
                    }
                }

                if let Some(source_skill) = skill.source_skills.iter().flatten().min().and_then(|id| SKILL_DATA.get(id)) {
                    return SkillSlim { 
                        id: source_skill.id as u32,
                        parent_id: Some(skill_id),
                        is_hyper_awakening: source_skill.is_hyper_awakening,
                        name: source_skill.name.clone().unwrap_or_default(),
                        icon: source_skill.icon.clone().unwrap_or_default(),
                        ..Default::default()
                    }
                }

                if skill.source_skills.is_some() {
                    return SkillSlim { 
                        id: skill_id,
                        parent_id: None,
                        name: default_skill_name,
                        icon: default_icon,
                        ..Default::default()
                    }
                }

                return SkillSlim { 
                    id: skill_id,
                    parent_id: None,
                    is_hyper_awakening: skill.is_hyper_awakening,
                    name: skill.name.clone().unwrap_or_default(),
                    icon: skill.icon.clone().unwrap_or_default(),
                    ..Default::default()
                }
            }

            let relative_skill_id = skill_id - (skill_id % 10);
            if let Some(skill) = SKILL_DATA.get(&relative_skill_id) {
                return SkillSlim { 
                    id: skill.id as u32,
                    parent_id: Some(skill_id),
                    is_hyper_awakening: skill.is_hyper_awakening,
                    name: skill.name.clone().unwrap_or_default(),
                    icon: skill.icon.clone().unwrap_or_default(),
                    ..Default::default()
                }
            }

            SkillSlim { id: skill_id, parent_id: None, name: default_skill_name, icon: default_icon, ..Default::default() }
        },
        (Some(skill_id), Some(skill_effect_id)) => { panic!("Unhandled {skill_id} {skill_effect_id}"); }
    }
}


pub fn damage_gem_value_to_level(value: u32, tier: u8) -> u8 {
    if tier == 4 {
        match value {
            4400 => 10,
            4000 => 9,
            3600 => 8,
            3200 => 7,
            2800 => 6,
            2400 => 5,
            2000 => 4,
            1600 => 3,
            1200 => 2,
            800 => 1,
            _ => 0,
        }
    } else {
        match value {
            4000 => 10,
            3000 => 9,
            2400 => 8,
            2100 => 7,
            1800 => 6,
            1500 => 5,
            1200 => 4,
            900 => 3,
            600 => 2,
            300 => 1,
            _ => 0,
        }
    }
}

pub fn cooldown_gem_value_to_level(value: u32, tier: u8) -> u8 {
    if tier == 4 {
        match value {
            2400 => 10,
            2200 => 9,
            2000 => 8,
            1800 => 7,
            1600 => 6,
            1400 => 5,
            1200 => 4,
            1000 => 3,
            800 => 2,
            600 => 1,
            _ => 0,
        }
    } else {
        match value {
            2000 => 10,
            1800 => 9,
            1600 => 8,
            1400 => 7,
            1200 => 6,
            1000 => 5,
            800 => 4,
            600 => 3,
            400 => 2,
            200 => 1,
            _ => 0,
        }
    }
}

pub fn support_damage_gem_value_to_level(value: u32) -> u8 {
    match value {
        1000 => 10,
        900 => 9,
        800 => 8,
        700 => 7,
        600 => 6,
        500 => 5,
        400 => 4,
        300 => 3,
        200 => 2,
        100 => 1,
        _ => 0,
    }
}

pub fn gem_skill_id_to_skill_ids(skill_id: u32) -> Vec<u32> {
    match skill_id {
        13000 | 13001 => vec![18011, 18030], // destroyer hypergravity skills
        23000 => vec![
            20311, 20310, 20070, 20071, 20080, 20081, 20170, 20181, 20280, 20281,
        ], // summoner elemental damage
        41000 => vec![25038, 25035, 25036, 25037, 25400, 25401, 25402], // db surge skill
        42000 | 42001 => vec![
            27800, 27030, 27810, 27820, 27830, 27840, 27850, 27860, 27940, 27960,
        ], // sh transformation skills
        51001 => vec![28159, 28160, 28161, 28162, 28170], // sharpshooter bird skill
        53000 | 53001 => vec![30240, 30250, 30260, 30270, 30290], // arty barrage skills
        54000 | 54001 => vec![
            35720, 35750, 35760, 35761, 35770, 35771, 35780, 35781, 35790, 35800,
        ], // machinist transformation skills
        62000 => vec![32040, 32041],         // aeromancer sun shower
        24000 => vec![
            21140, 21141, 21142, 21143, 21130, 21131, 21132, 21133, // bard serenade skills
            21147, // bard tempest
        ],
        47000 => vec![47950], // bk breaker identity
        60000 => vec![
            31050, 31051, 31110, 31120, 31121, 31130, 31131, 31140, 31141, // artist moonfall
            31145, // artist rising moon
        ],
        19030 => vec![19290, 19030, 19300],  // arcana evokes
        63000 | 63001 => vec![33200, 33201], // wildsoul swish bear
        63002 | 63003 => vec![33230, 33231], // wildsoul boulder bear
        63004 | 63005 => vec![33330, 33331], // wildsoul fox leap
        63006 | 63007 => vec![33320, 33321], // wildsoul fox flame
        63008 | 63009 => vec![33400, 33410], // wildsoul identity skills
        _ => vec![skill_id],
    }
}

pub fn get_engravings(engraving_ids: &Option<Vec<u32>>) -> Option<Vec<String>> {
    let ids = match engraving_ids {
        Some(engravings) => engravings,
        None => return None,
    };
    let mut engravings: Vec<String> = Vec::new();

    for engraving_id in ids.iter() {
        if let Some(engraving_data) = ENGRAVING_DATA.get(engraving_id) {
            engravings.push(engraving_data.name.clone().unwrap_or("Unknown".to_string()));
        }
    }

    engravings.sort_unstable();
    Some(engravings)
}

pub fn is_hat_buff(buff_id: &u32) -> bool {
    matches!(buff_id, 362600 | 212305 | 319503)
}

pub fn generate_intervals(start: i64, end: i64) -> Vec<i64> {
    if start >= end {
        return Vec::new();
    }

    (0..end - start).step_by(1_000).collect()
}

pub fn sum_in_range(vec: &[(i64, i64)], start: i64, end: i64) -> i64 {
    let start_idx = binary_search_left(vec, start);
    let end_idx = binary_search_left(vec, end + 1);

    vec[start_idx..end_idx]
        .iter()
        .map(|&(_, second)| second)
        .sum()
}

fn binary_search_left(vec: &[(i64, i64)], target: i64) -> usize {
    let mut left = 0;
    let mut right = vec.len();

    while left < right {
        let mid = left + (right - left) / 2;
        match vec[mid].0.cmp(&target) {
            Ordering::Less => left = mid + 1,
            _ => right = mid,
        }
    }

    left
}

pub fn calculate_average_dps(data: &[(i64, i64)], start_time: i64, end_time: i64) -> Vec<i64> {
    let step = 5;
    let mut results = vec![0; ((end_time - start_time) / step + 1) as usize];
    let mut current_sum = 0;
    let mut data_iter = data.iter();
    let mut current_data = data_iter.next();

    for t in (start_time..=end_time).step_by(step as usize) {
        while let Some((timestamp, value)) = current_data {
            if *timestamp / 1000 <= t {
                current_sum += value;
                current_data = data_iter.next();
            } else {
                break;
            }
        }

        results[((t - start_time) / step) as usize] = current_sum / (t - start_time + 1);
    }

    results
}

pub fn check_tripod_index_change(before: Option<TripodIndex>, after: Option<TripodIndex>) -> bool {
    if before.is_none() && after.is_none() {
        return false;
    }

    if before.is_none() || after.is_none() {
        return true;
    }

    let before = before.unwrap();
    let after = after.unwrap();

    before != after
}

pub fn check_tripod_level_change(before: Option<TripodLevel>, after: Option<TripodLevel>) -> bool {
    if before.is_none() && after.is_none() {
        return false;
    }

    if before.is_none() || after.is_none() {
        return true;
    }

    let before = before.unwrap();
    let after = after.unwrap();

    before != after
}

pub fn map_status_effect(se: &StatusEffectDetails) -> u32 {
    if se.custom_id > 0 {
        se.custom_id
    } else {
        se.status_effect_id
    }
}

pub fn is_valid_player(player: &EncounterEntity) -> bool {
    player.gear_score >= 0.0
        && player.entity_type == EntityType::Player
        && player.character_id != 0
        && player.class_id != 0
        && player.name != "You"
        && player
            .name
            .chars()
            .next()
            .unwrap_or_default()
            .is_uppercase()
}

pub fn get_new_id(source_skill: u32) -> u32 {
    source_skill + 1_000_000_000
}

pub fn get_skill_id(new_skill: u32, original_buff_id: u32) -> u32 {
    new_skill - 1_000_000_000 - original_buff_id
}

pub fn get_main_boss_name(boss_name: &str) -> String {
    match boss_name {
        "Chaos Lightning Dragon Jade" => "Argeos",
        "Vicious Argeos" | "Ruthless Lakadroff" | "Untrue Crimson Yoho" | "Despicable Skolakia" => {
            "Behemoth, the Storm Commander"
        }
        _ => boss_name,
    }
    .to_string()
}

pub fn get_player_spec(player: &EncounterEntity, buffs: &HashMap<u32, StatusEffect>) -> String {
    if player.skills.len() < 8 {
        return "Unknown".to_string();
    }

    match player.class.as_str() {
        "Berserker" => {
            if player.skills.contains_key(&16140) {
                "Berserker Technique"
            } else {
                "Mayhem"
            }
        }
        "Destroyer" => {
            if player.skills.contains_key(&18090) {
                "Gravity Training"
            } else {
                "Rage Hammer"
            }
        }
        "Gunlancer" => {
            if player.skills.contains_key(&17200) && player.skills.contains_key(&17210) {
                "Lone Knight"
            } else if player.skills.contains_key(&17140) && player.skills.contains_key(&17110) {
                "Combat Readiness"
            } else {
                "Princess"
            }
        }
        "Paladin" => {
            // contains one of light shock, sword of justice, god's decree, or holy explosion
            if (player.skills.contains_key(&36050)
                || player.skills.contains_key(&36080)
                || player.skills.contains_key(&36150)
                || player.skills.contains_key(&36100))
                && player.skills.contains_key(&36200)
                && player.skills.contains_key(&36170)
            {
                "Blessed Aura"
            } else {
                "Judgment"
            }
        }
        "Slayer" => {
            if player.skills.contains_key(&45004) {
                "Punisher"
            } else {
                "Predator"
            }
        }
        "Arcanist" => {
            if player.skills.contains_key(&19282) {
                "Order of the Emperor"
            } else {
                "Grace of the Empress"
            }
        }
        "Summoner" => {
            if player
                .skills
                .iter()
                .any(|(_, skill)| skill.name.contains("Kelsion"))
            {
                "Communication Overflow"
            } else {
                "Master Summoner"
            }
        }
        "Bard" => {
            // contains one of guardian tune, rhapsody of light, or wind of music
            if (player.skills.contains_key(&21250)
                || player.skills.contains_key(&21260)
                || player.skills.contains_key(&21070))
                && player.skills.contains_key(&21160)
            {
                "Desperate Salvation"
            } else {
                "True Courage"
            }
        }
        "Sorceress" => {
            // if has arcane rupture
            if player.skills.contains_key(&37100) || player.skills.contains_key(&37101) {
                "Igniter"
            } else {
                "Reflux"
            }
        }
        "Wardancer" => {
            if player.skills.contains_key(&22340) {
                "Esoteric Skill Enhancement"
            } else {
                "First Intention"
            }
        }
        "Scrapper" => {
            if player.skills.contains_key(&23230) {
                "Ultimate Skill: Taijutsu"
            } else {
                "Shock Training"
            }
        }
        "Soulfist" => {
            if player.skills.contains_key(&24200) {
                "Energy Overflow"
            } else {
                "Robust Spirit"
            }
        }
        "Glaivier" => {
            if player.skills.contains_key(&34590) {
                "Pinnacle"
            } else {
                "Control"
            }
        }
        "Striker" => {
            if player.skills.contains_key(&39290) {
                "Deathblow"
            } else {
                "Esoteric Flurry"
            }
        }
        "Breaker" => {
            if player.skills.contains_key(&47020) {
                "Asura's Path"
            } else {
                "Brawl King Storm"
            }
        }
        "Deathblade" => {
            if player.skills.contains_key(&25038) {
                "Surge"
            } else {
                "Remaining Energy"
            }
        }
        "Shadowhunter" => {
            if player.skills.contains_key(&27860) {
                "Demonic Impulse"
            } else {
                "Perfect Suppression"
            }
        }
        "Reaper" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names.iter().any(|s| s.contains("Lunar Voice")) {
                "Lunar Voice"
            } else {
                "Hunger"
            }
        }
        "Souleater" => {
            if player.skills.contains_key(&46250) {
                "Night's Edge"
            } else {
                "Full Moon Harvester"
            }
        }
        "Sharpshooter" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names
                .iter()
                .any(|s| s.contains("Loyal Companion") || s.contains("Hawk Support"))
            {
                "Loyal Companion"
            } else {
                "Death Strike"
            }
        }
        "Deadeye" => {
            if player.skills.contains_key(&29300) {
                "Enhanced Weapon"
            } else {
                "Pistoleer"
            }
        }
        "Artillerist" => {
            if player.skills.contains_key(&30260) {
                "Barrage Enhancement"
            } else {
                "Firepower Enhancement"
            }
        }
        "Machinist" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names
                .iter()
                .any(|s| s.contains("Combat Mode") || s.contains("Evolutionary Legacy"))
            {
                "Evolutionary Legacy"
            } else {
                "Arthetinean Skill"
            }
        }
        "Gunslinger" => {
            if player.skills.contains_key(&38110) {
                "Peacemaker"
            } else {
                "Time to Hunt"
            }
        }
        "Artist" => {
            // contains one of drawing orchids, starry night, or illusion door
            // and doesn't contain cattle drive, dps skill
            if (player.skills.contains_key(&31420)
                || player.skills.contains_key(&31450)
                || player.skills.contains_key(&31220))
                && player.skills.contains_key(&31400)
                && player.skills.contains_key(&31410)
                && !player.skills.contains_key(&31940)
            {
                "Full Bloom"
            } else {
                "Recurrence"
            }
        }
        "Aeromancer" => {
            if player.skills.contains_key(&32250) && player.skills.contains_key(&32260) {
                "Wind Fury"
            } else {
                "Drizzle"
            }
        }
        "Wildsoul" => {
            if player.skills.contains_key(&33400) || player.skills.contains_key(&33410) {
                "Ferality"
            } else {
                "Phantom Beast Awakening"
            }
        }
        _ => "Unknown",
    }
    .to_string()
}

fn get_buff_names(player: &EncounterEntity, buffs: &HashMap<u32, StatusEffect>) -> Vec<String> {
    let mut names = Vec::new();
    for (id, _) in player.damage_stats.buffed_by.iter() {
        if let Some(buff) = buffs.get(id) {
            names.push(buff.source.name.clone());
        }
    }

    names
}

pub fn get_spec_from_ark_passive(node: &ArkPassiveNode) -> String {
    match node.id {
        2160000 => "Berserker Technique",
        2160010 => "Mayhem",
        2170000 => "Lone Knight",
        2170010 => "Combat Readiness",
        2180000 => "Rage Hammer",
        2180010 => "Gravity Training",
        2360000 => "Judgment",
        2360010 => "Blessed Aura",
        2450000 => "Punisher",
        2450010 => "Predator",
        2230000 => "Ultimate Skill: Taijutsu",
        2230100 => "Shock Training",
        2220000 => "First Intention",
        2220100 => "Esoteric Skill Enhancement",
        2240000 => "Energy Overflow",
        2240100 => "Robust Spirit",
        2340000 => "Control",
        2340100 => "Pinnacle",
        2470000 => "Brawl King Storm",
        2470100 => "Asura's Path",
        2390000 => "Esoteric Flurry",
        2390010 => "Deathblow",
        2300000 => "Barrage Enhancement",
        2300100 => "Firepower Enhancement",
        2290000 => "Enhanced Weapon",
        2290100 => "Pistoleer",
        2280000 => "Death Strike",
        2280100 => "Loyal Companion",
        2350000 => "Evolutionary Legacy",
        2350100 => "Arthetinean Skill",
        2380000 => "Peacemaker",
        2380100 => "Time to Hunt",
        2370000 => "Igniter",
        2370100 => "Reflux",
        2190000 => "Grace of the Empress",
        2190100 => "Order of the Emperor",
        2200000 => "Communication Overflow",
        2200100 => "Master Summoner",
        2210000 => "Desperate Salvation",
        2210100 => "True Courage",
        2270000 => "Demonic Impulse",
        2270600 => "Perfect Suppression",
        2250000 => "Surge",
        2250600 => "Remaining Energy",
        2260000 => "Lunar Voice",
        2260600 => "Hunger",
        2460000 => "Full Moon Harvester",
        2460600 => "Night's Edge",
        2320000 => "Wind Fury",
        2320600 => "Drizzle",
        2310000 => "Full Bloom",
        2310600 => "Recurrence",
        2330000 => "Ferality",
        2330100 => "Phantom Beast Awakening",
        _ => "Unknown",
    }
    .to_string()
}

pub fn boss_to_raid_map(boss: &str, max_hp: i64) -> Option<String> {
    match boss {
        "Phantom Legion Commander Brelshaza" => {
            if max_hp > 100_000_000_000 {
                Some("Act 2: Brelshaza G2".to_string())
            } else {
                Some("Brelshaza G6".to_string())
            }
        }
        _ => RAID_MAP.get(boss).cloned(),
    }
}

pub fn get_current_and_max_hp(stat_pair: &Vec<StatPair>) -> (i64, i64) {
    let mut hp: Option<i64> = None;
    let mut max_hp: Option<i64> = None;

    for pair in stat_pair {
        match pair.stat_type as u32 {
            1 => hp = Some(pair.value),
            27 => max_hp = Some(pair.value),
            _ => {}
        }
        if hp.is_some() && max_hp.is_some() {
            break;
        }
    }

    (hp.unwrap_or_default(), max_hp.unwrap_or_default())
}

pub fn truncate_gear_level(gear_level: f32) -> f32 {
    f32::trunc(gear_level * 100.) / 100.
}

pub fn is_valid_for_raid(status_effect: &StatusEffectDetails) -> bool {
    (status_effect.buff_category == StatusEffectBuffCategory::BattleItem
        || status_effect.buff_category == StatusEffectBuffCategory::Bracelet
        || status_effect.buff_category == StatusEffectBuffCategory::Elixir
        || status_effect.buff_category == StatusEffectBuffCategory::Etc)
        && status_effect.category == StatusEffectCategory::Debuff
        && status_effect.show_type == StatusEffectShowType::All
}

pub fn build_status_effect(
    se_data: StatusEffectData,
    target_id: u64,
    source_id: u64,
    target_type: StatusEffectTargetType,
    timestamp: DateTime<Utc>,
) -> StatusEffectDetails {
    let value = get_status_effect_value(&se_data.value.bytearray_0);
    let mut status_effect_category = StatusEffectCategory::Other;
    let mut buff_category = StatusEffectBuffCategory::Other;
    let mut show_type = StatusEffectShowType::Other;
    let mut status_effect_type = StatusEffectType::Other;
    let mut name = "Unknown".to_string();
    let mut db_target_type = "".to_string();
    let mut source_skills = vec![];

    if let Some(effect) = SKILL_BUFF_DATA.get(&se_data.status_effect_id) {
        source_skills = effect.source_skills.clone().unwrap_or_default();

        name = effect.name.clone().unwrap_or_default();
        if effect.category.as_str() == "debuff" {
            status_effect_category = StatusEffectCategory::Debuff
        }
        match effect.buff_category.clone().unwrap_or_default().as_str() {
            "bracelet" => buff_category = StatusEffectBuffCategory::Bracelet,
            "etc" => buff_category = StatusEffectBuffCategory::Etc,
            "battleitem" => buff_category = StatusEffectBuffCategory::BattleItem,
            "elixir" => buff_category = StatusEffectBuffCategory::Elixir,
            _ => {}
        }
        if effect.icon_show_type.clone().unwrap_or_default() == "all" {
            show_type = StatusEffectShowType::All
        }
        status_effect_type = match effect.buff_type.as_str() {
            "shield" => StatusEffectType::Shield,
            "freeze" | "fear" | "stun" | "sleep" | "earthquake" | "electrocution"
            | "polymorph_pc" | "forced_move" | "mind_control" | "paralyzation" => {
                StatusEffectType::HardCrowdControl
            }
            _ => StatusEffectType::Other,
        };
        db_target_type = effect.target.to_string();
    }

    let expiry = if se_data.total_time > 0. && se_data.total_time < 604800. {
        Some(
            timestamp
                + Duration::milliseconds((se_data.total_time as i64) * 1000 + TIMEOUT_DELAY_MS),
        )
    } else {
        None
    };

    StatusEffectDetails {
        source_skills,
        instance_id: se_data.status_effect_instance_id,
        source_id,
        target_id,
        status_effect_id: se_data.status_effect_id,
        custom_id: 0,
        target_type,
        db_target_type,
        value,
        stack_count: se_data.stack_count,
        buff_category,
        category: status_effect_category,
        status_effect_type,
        show_type,
        expiration_delay: se_data.total_time,
        expire_at: expiry,
        end_tick: se_data.end_tick,
        name,
        timestamp,
    }
}

pub fn get_status_effect_value(value: &Option<Vec<u8>>) -> u64 {
    value.as_ref().map_or(0, |v| {
        let c1 = v
            .get(0..8)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        let c2 = v
            .get(8..16)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        c1.min(c2)
    })
}

pub fn is_active(e: &EncounterEntity, local_player: &str) -> bool {
    ((e.entity_type == EntityType::Player && e.class_id > 0)
        || e.name == local_player
        || e.entity_type == EntityType::Esther
        || (e.entity_type == EntityType::Boss && e.max_hp > 0))
        && e.damage_stats.damage_dealt > 0
}