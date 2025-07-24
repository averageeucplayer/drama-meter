use std::mem::transmute;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{cell::RefCell, fmt::Debug};
use std::any::type_name;
use chrono::{DateTime, Utc};
use hashbrown::HashMap;
use log::*;
use meter_core::decryption::DamageEncryptionHandler;
use meter_core::packets::structures::NpcStruct;
use meter_core::packets::{definitions::*, opcodes::Pkt};
use tauri::{AppHandle, Emitter, EventTarget, Manager};
use anyhow::Result;
use tokio::task;

use crate::core::stats_api::StatsApi;
use crate::misc::data::VALID_ZONES;
use crate::database::{Database, SaveToDb};
use crate::entity::{player, EntityVariant};
use crate::core::encounter_state::EncounterState;
use crate::models::{RaidDifficulty, StatusEffectTargetType, StatusEffectType};
use crate::core::utils::*;
use crate::misc::local::LocalManager;
use crate::models::{DamageData, EntityType, LocalInfo, LocalPlayer, SkillCast};
use crate::misc::region::RegionManager;
use crate::models::TripodIndex;
use crate::models::TripodLevel;

pub fn handle(
    recorded_on: DateTime<Utc>,
    op: Pkt,
    app: AppHandle,
    state: &mut EncounterState,
    data: &[u8],
    damage_handler: &DamageEncryptionHandler,
    local_manager: &mut LocalManager,
    region_manager: &mut RegionManager,
    stats_api: &Arc<StatsApi>,
    database: Arc<Database>
) -> Result<()> {
    match op {
        Pkt::CounterAttackNotify => {
            let PKTCounterAttackNotify { source_id } = PKTCounterAttackNotify::new(data)?;
            
            if let Some(entity) = state.get_player_by_id(source_id) {
                entity.encounter_stats.skill_stats.counters += 1;
            }
        }
        Pkt::DeathNotify => {
            let PKTDeathNotify { target_id } = PKTDeathNotify::new(data)?;
            
            if let Some(entity) = state.get_entity_mut(&target_id) {
                match entity {
                    EntityVariant::Player(player) => {
                        let death_log = &mut player.encounter_stats.death_log;
                        death_log.count += 1;
                        death_log.recorded_on = recorded_on;
                        player.trim_incapacitations_to_death_time();
                    }
                    EntityVariant::Boss(boss) => {
                        state.boss_dead_update = true;
                    }
                    _ => {}
                }
            }
        }
        Pkt::InitEnv => {
            let PKTInitEnv { player_id} = PKTInitEnv::new(data)?;
            
            state.raid_difficulty = RaidDifficulty::Unknown;
            state.damage_is_valid = true;

            state.init_env(recorded_on, player_id);

            if !state.saved && let Some(model) = state.get_encounter(false) {
                let app = app.clone();

                save_to_db(app.clone(), stats_api.clone(), database.clone(), model);
            }

            state.soft_reset(false);

            app.emit_to(EventTarget::Any, "zone-change", "")?;

            state.valid_zone = false;
            state.region = region_manager.get();

            info!("region: {:?}", state.region);
        }
        Pkt::InitPC => {
            
            let PKTInitPC {
                player_id,
                name,
                character_id,
                class_id,
                gear_level,
                stat_pairs,
                status_effect_datas,
            } = PKTInitPC::new(data)?;

            let (hp, max_hp) = get_current_and_max_hp(&stat_pairs);
            state.init_pc(
                recorded_on,
                player_id,
                name.clone(),
                character_id,
                class_id,
                gear_level,
                stat_pairs,
                status_effect_datas,
                hp,
                max_hp
            );

            local_manager.write(name, character_id, recorded_on);
        }
        Pkt::NewPC => {
            let PKTNewPC {
                pc_struct: PKTNewPCInner {
                    character_id,
                    class_id,
                    player_id,
                    name,
                    max_item_level,
                    stat_pairs,
                    equip_item_datas,
                    status_effect_datas
                }
            } = PKTNewPC::new(data)?;
            let (current_hp, max_hp) = get_current_and_max_hp(&stat_pairs);
            state.new_pc(
                recorded_on,
                player_id,
                name,
                character_id,
                class_id,
                max_item_level,
                current_hp,
                max_hp,
                stat_pairs,
                status_effect_datas
            );
        }
        Pkt::NewNpc => {
            let PKTNewNpc {
                npc_struct: NpcStruct {
                    balance_level,
                    level,
                    object_id,
                    type_id,
                    stat_pairs,
                    status_effect_datas,
                }
            } = PKTNewNpc::new(data)?;
            let (hp, max_hp) = get_current_and_max_hp(&stat_pairs);
            state.new_npc(
                recorded_on,
                object_id,
                type_id,
                level,
                balance_level.value.unwrap_or(level),
                max_hp,
                stat_pairs,
                status_effect_datas
            );

        }
        Pkt::NewNpcSummon => {
            let PKTNewNpcSummon {
                npc_struct: NpcStruct {
                    balance_level,
                    level,
                    object_id,
                    type_id,
                    stat_pairs,
                    status_effect_datas
                },
                owner_id
            } = PKTNewNpcSummon::new(data)?;
            let (hp, max_hp) = get_current_and_max_hp(&stat_pairs);
            state.new_npc_summon(
                recorded_on,
                object_id,
                type_id,
                owner_id,
                level,
                balance_level.value.unwrap_or(level),
                max_hp,
                stat_pairs,
                status_effect_datas
            );
        }
        Pkt::NewProjectile => {
            let packet = PKTNewProjectile::new(data)?;
            let PKTNewProjectile {
                projectile_info: PKTNewProjectileInner {
                    owner_id,
                    projectile_id,
                    skill_id,
                    skill_effect
                }
            } = packet;
            state.new_projectile(owner_id, projectile_id, skill_id, skill_effect, recorded_on);

            if state.is_player(owner_id) && skill_id > 0
            {
                let key = (owner_id, skill_id);
                if let Some(timestamp) = state.skill_timestamp.get(&key) {
                    state.projectile_id_to_timestamp.insert(projectile_id, timestamp);
                }
            }
        }
        Pkt::NewTrap => {
            let PKTNewTrap {
                trap_struct: PKTNewTrapInner {
                    object_id,
                    owner_id,
                    skill_effect,
                    skill_id
                }
            } = PKTNewTrap::new(data)?;
            state.new_trap(object_id, owner_id, skill_id, skill_effect, recorded_on);

            if state.is_player(owner_id) && skill_id > 0
            {
                let key = (owner_id, skill_id);
                if let Some(timestamp) = state.skill_timestamp.get(&key) {
                    state.projectile_id_to_timestamp
                        .insert(object_id, timestamp);
                }
            }
        }

        Pkt::RaidBegin => {
            let PKTRaidBegin { raid_id } = PKTRaidBegin::new(data)?;
            info!("raid begin: {}", raid_id);

            match raid_id {
                308226 | 308227 | 308239 | 308339 => {
                    state.raid_difficulty = RaidDifficulty::Trial
                }
                308428 | 308429 | 308420 | 308410 | 308411 | 308414 | 308422 | 308424
                | 308421 | 308412 | 308423 | 308426 | 308416 | 308419 | 308415 | 308437
                | 308417 | 308418 | 308425 | 308430 => {
                    state.raid_difficulty = RaidDifficulty::Challenge
                }
                _ => {
                    state.raid_difficulty = RaidDifficulty::Unknown
                }
            }

            state.valid_zone = VALID_ZONES.contains(&raid_id);
        }
        Pkt::RaidBossKillNotify => {

            app.emit_to(EventTarget::Any, "phase-transition", 1)?;

            state.raid_clear = true;

            info!("phase: 1 - RaidBossKillNotify");
        }
        Pkt::RaidResult => {
            
            state.party_freeze = true;
            state.party_info = state.get_party();

            app.emit_to(EventTarget::Any, "phase-transition", 0)?;

            if let Some(encounter) = state.get_encounter(false) {
                state.valid_zone = false;
                
                save_to_db(app.clone(), stats_api.clone(), database, encounter);
                state.saved = true;
            }

            state.is_resetting = true;
            state.raid_end_cd = recorded_on;
            info!("phase: 0 - RaidResult");
        }
        Pkt::RemoveObject => {
            let PKTRemoveObject { unpublished_objects} = PKTRemoveObject::new(data)?;
            
            state.on_remove_objects(unpublished_objects.into_iter().map(|pr| pr.object_id).collect());
        }
        Pkt::SkillCastNotify => {
            let PKTSkillCastNotify { skill_id, source_id } = PKTSkillCastNotify::new(data)?;
            
            state.promote_to_player(recorded_on, source_id, skill_id);

            if let Some(player) = state.get_player_by_id(source_id).filter(|pr| pr.class_id == 202 ) {
                unsafe { state.on_skill_start(
                    source_id,
                    skill_id,
                    None,
                    None,
                    recorded_on
                ) };
            }
        }
        Pkt::SkillStartNotify => {
            let PKTSkillStartNotify {
                skill_id,
                skill_option_data,
                source_id
            } = PKTSkillStartNotify::new(data)?;
            
            if !state.has_fight_started() {
                return Ok(())
            }

            state.promote_to_player(recorded_on, source_id, skill_id);
            
            let tripod_index = skill_option_data
                    .tripod_index
                    .map(|tripod_index| TripodIndex {
                        first: tripod_index.first,
                        second: tripod_index.second,
                        third: tripod_index.third,
                    });
            let tripod_level = skill_option_data
                    .tripod_level
                    .map(|tripod_level| TripodLevel {
                        first: tripod_level.first,
                        second: tripod_level.second,
                        third: tripod_level.third,
                    });

            unsafe { state.on_skill_start(
                source_id,
                skill_id,
                tripod_index,
                tripod_level,
                recorded_on,
            ) };
        }
        Pkt::SkillDamageAbnormalMoveNotify => {
            if state.has_restarted(recorded_on) {
                info!("ignoring damage - SkillDamageAbnormalMoveNotify");

                return Ok(())
            }
            let PKTSkillDamageAbnormalMoveNotify {
                skill_damage_abnormal_move_events,
                skill_effect_id,
                skill_id,
                source_id
            } = PKTSkillDamageAbnormalMoveNotify::new(data)?;
            
            let owner = state.get_source_entity(source_id, recorded_on);
            let boss_only_damage = state.boss_only_damage;
            let self_ptr = state as *mut EncounterState;

            for mut event in skill_damage_abnormal_move_events.into_iter() {
                let target_id = event.skill_damage_event.target_id;

                if !damage_handler.decrypt_damage_event(&mut event.skill_damage_event) {
                    state.damage_is_valid = false;
                    continue;
                }

                let target_entity = unsafe { (*self_ptr).get_or_create_entity(target_id, recorded_on) };
                let source_entity = unsafe { (*self_ptr).get_or_create_entity(source_id, recorded_on) };

                // track potential knockdown
                if let Some(player) = target_entity.as_player_mut() {
                    player.on_abnormal_move(recorded_on, &event.skill_move_option_data);
                }

                let damage_data = DamageData::from(
                    state.is_initial(),
                    boss_only_damage,
                    source_id,
                    target_entity.id(),
                    recorded_on,
                    (skill_id != 0).then(|| skill_id),
                    (skill_effect_id != 0).then(|| skill_effect_id),
                    event.skill_damage_event
                );

                if let Some(damage_data) = damage_data {

                    if damage_data.is_initial {
                        state.started_on = recorded_on;
                        app.emit_to(EventTarget::Any, "raid-start", recorded_on.timestamp_millis())?;
                    }

                    state.on_damage(damage_data, source_entity, target_entity);
                }
            }
        }
        Pkt::SkillDamageNotify => {
            if state.has_restarted(recorded_on) {

                info!("ignoring damage - SkillDamageNotify");
                return Ok(())
            }

            let PKTSkillDamageNotify {
                skill_damage_events,
                skill_effect_id,
                skill_id,
                source_id
            } = PKTSkillDamageNotify::new(data)?;
            
            let owner = state.get_source_entity(source_id, recorded_on);
            let boss_only_damage = state.boss_only_damage;
            let self_ptr = state as *mut EncounterState;

            for mut event in skill_damage_events.into_iter() {
                let target_id = event.target_id;

                if !damage_handler.decrypt_damage_event(&mut event) {
                    state.damage_is_valid = false;
                    continue;
                }

                let target_entity = unsafe { (*self_ptr).get_or_create_entity(target_id, recorded_on) };
                let source_entity = unsafe { (*self_ptr).get_or_create_entity(source_id, recorded_on) };
                
                let damage_data = DamageData::from(
                    state.is_initial(),
                    boss_only_damage,
                    source_id,
                    target_entity.id(),
                    recorded_on,
                    (skill_id != 0).then(|| skill_id),
                    skill_effect_id,
                    event
                );

                if let Some(damage_data) = damage_data {

                    if damage_data.is_initial {
                        state.started_on = recorded_on;
                        app.emit_to(EventTarget::Any, "raid-start", recorded_on.timestamp_millis())?;
                    }

                    state.on_damage(damage_data, source_entity, target_entity);
                }
            }
        }
        Pkt::PartyInfo => {
            let PKTPartyInfo {
                party_instance_id,
                party_member_datas,
                raid_instance_id
            } = PKTPartyInfo::new(data)?;

            state.party_info(
                party_instance_id,
                raid_instance_id,
                party_member_datas,
                local_manager.get(),
                recorded_on
            );
        }
        Pkt::PartyLeaveResult => {
            let packet = PKTPartyLeaveResult::new(data)?;
         
            // state.remove(packet.party_instance_id, packet.name);
        }
        Pkt::PartyStatusEffectAddNotify => {
            let PKTPartyStatusEffectAddNotify {
                character_id,
                status_effect_datas
            } = PKTPartyStatusEffectAddNotify::new(data)?;

            unsafe { state.party_status_effect_add(recorded_on, character_id, status_effect_datas) };
        }
        Pkt::PartyStatusEffectRemoveNotify => {
            let PKTPartyStatusEffectRemoveNotify { 
                character_id,
                reason,
                status_effect_instance_ids: instance_ids
            } = PKTPartyStatusEffectRemoveNotify::new(data)?;

            state.on_party_status_effects_remove(character_id, instance_ids, reason, recorded_on);
        }
        Pkt::PartyStatusEffectResultNotify => {
            let PKTPartyStatusEffectResultNotify {
                character_id,
                party_instance_id,
                raid_instance_id
            } = PKTPartyStatusEffectResultNotify::new(data)?;
           
            state.update_player(character_id, party_instance_id, raid_instance_id);
        }
        Pkt::StatusEffectAddNotify => {
            let PKTStatusEffectAddNotify {
                object_id,
                status_effect_data
            } = PKTStatusEffectAddNotify::new(data)?;

            state.on_status_effect_add(&status_effect_data, object_id, recorded_on);
        }
        Pkt::StatusEffectRemoveNotify => {
            let PKTStatusEffectRemoveNotify {
                object_id,
                reason,
                status_effect_instance_ids: instance_ids
            } = PKTStatusEffectRemoveNotify::new(data)?;

            state.on_status_effect_remove(object_id, reason, instance_ids, recorded_on);
        }
        Pkt::TriggerBossBattleStatus => {
            
            if state.is_saydon_glitch() {
                app.emit_to(EventTarget::Any, "phase-transition", 3)?;
                
                if let Some(model) = state.get_encounter(false) {
                    save_to_db(app.clone(), stats_api.clone(), database, model);
                    state.saved = true;
                }
                
                state.is_resetting = true;

                info!(
                    "phase: 3 - resetting encounter - TriggerBossBattleStatus"
                );
            }
        }
        Pkt::TriggerStartNotify => {
            let packet = PKTTriggerStartNotify::new(data)?;
            
            match packet.signal {
                57 | 59 | 61 | 63 | 74 | 76 => {
                    state.party_freeze = true;
                    state.party_info = state.get_party();
                    state.raid_clear = true;

                    app.emit_to(EventTarget::Any, "phase-transition", 2)?;

                    if let Some(model) = state.get_encounter(false) {
                    
                        save_to_db(app.clone(), stats_api.clone(), database.clone(), model);
                        state.saved = true;
                    }
                    
                    state.is_resetting = true;

                    state.raid_end_cd = recorded_on;
                    info!("phase: 2 - clear - TriggerStartNotify");
                }
                58 | 60 | 62 | 64 | 75 | 77 => {
                    state.party_freeze = true;
                    state.party_info = state.get_party();
                    state.raid_clear = false;

                    app.emit_to(EventTarget::Any, "phase-transition", 4)?;
        
                    if let Some(model) = state.get_encounter(false) {  
                        save_to_db(app.clone(), stats_api.clone(), database.clone(), model);
                        state.saved = true;
                    }
                    
                    state.is_resetting  = true;
                    state.raid_end_cd = recorded_on;
                    info!("phase: 4 - wipe - TriggerStartNotify");
                }
                27 | 10 | 11 => {
                    // debug_print(format_args!("old rdps sync time - {}", packet.trigger_signal_type));
                }
                _ => {}
            }
        }
        Pkt::ZoneMemberLoadStatusNotify => {
            let PKTZoneMemberLoadStatusNotify { zone_id, zone_level } = PKTZoneMemberLoadStatusNotify::new(data)?;
            state.valid_zone = VALID_ZONES.contains(&zone_id);

            if state.raid_difficulty as u8 >= zone_id as u8
            {
                return Ok(())
            }

            info!("raid zone: {} level: {}", &zone_id, &zone_level);

            state.raid_difficulty = unsafe { std::mem::transmute(zone_level as u8) };
        }
        Pkt::ZoneObjectUnpublishNotify => {
            let PKTZoneObjectUnpublishNotify { object_id } = PKTZoneObjectUnpublishNotify::new(data)?;

            state.remove_local_object(object_id);
        }
        Pkt::StatusEffectSyncDataNotify => {
            let PKTStatusEffectSyncDataNotify {
                character_id,
                object_id,
                status_effect_instance_id,
                value
            } = PKTStatusEffectSyncDataNotify::new(data)?;

            state.sync_status_effect(
                status_effect_instance_id,
                character_id,
                object_id,
                value,
                recorded_on
            );
        }
        Pkt::TroopMemberUpdateMinNotify => {
            let PKTTroopMemberUpdateMinNotify { 
                character_id,
                cur_hp,
                max_hp,
                status_effect_datas
            } = PKTTroopMemberUpdateMinNotify::new(data)?;
                
            let target_id = if let Some(player) = state.get_player_by_character_id(character_id) {
                player.encounter_stats.current_hp = cur_hp;
                player.encounter_stats.max_hp = max_hp;
                Some(player.id)
            } else { None };

            let target_id = match target_id {
                Some(id) => id,
                None => return Ok(()),
            };

            for se in status_effect_datas.iter() {
                let value = get_status_effect_value(&se.value.bytearray_0);
                state.sync_status_effect(
                    se.status_effect_instance_id,
                    character_id,
                    target_id,
                    value,
                    recorded_on
                );
            }
        }
        Pkt::NewTransit => {
            let packet = PKTNewTransit::new(data)?;
            damage_handler.update_zone_instance_id(packet.channel_id);
        }
        _ => {}
    }
    
    Ok(())
}