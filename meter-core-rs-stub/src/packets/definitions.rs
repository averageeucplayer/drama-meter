use serde::{Deserialize, Serialize};
use bincode::{self, Decode, Encode};
use crate::types::*;
use super::{common::SkillMoveOptionData, structures::*};

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTCounterAttackNotify {
    pub source_id: u64
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTDeathNotify {
    pub target_id: u64
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTIdentityGaugeChangeNotify {
    pub player_id: EntityId,
    pub identity_gauge1: u32,
    pub identity_gauge2: u32,
    pub identity_gauge3: u32
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTInitEnv {
    pub player_id: EntityId
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTInitPC {
    pub player_id: EntityId,
    pub name: String,
    pub character_id: CharacterId,
    pub class_id: ClassId,
    pub gear_level: GearLevel,
    pub stat_pairs: Vec<StatPair>,
    pub status_effect_datas: Vec<StatusEffectData>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewNpc {
    pub npc_struct: NpcStruct
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewNpcSummon {
    pub owner_id: EntityId,
    pub npc_struct: NpcStruct
}


#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewProjectileInner {
    pub projectile_id: EntityId,
    pub owner_id: EntityId,
    pub skill_id: SkillId,
    pub skill_effect: SkillEffectId,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewProjectile {
    pub projectile_info: PKTNewProjectileInner
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewTrapInner {
    pub object_id: EntityId,
    pub owner_id: EntityId,
    pub skill_id: SkillId,
    pub skill_effect: SkillEffectId
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewTrap {
    pub trap_struct: PKTNewTrapInner
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTRaidBegin {
    pub raid_id: u32,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTRemoveObjectInner {
    pub object_id: EntityId
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTRemoveObject {
    pub unpublished_objects: Vec<PKTRemoveObjectInner>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTSkillCastNotify {
    pub source_id: EntityId,
    pub skill_id: SkillId,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone, Copy)]
pub struct TripodIndex {
    pub first: u8,
    pub second: u8,
    pub third: u8,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone, Copy)]
pub struct TripodLevel {
    pub first: u16,
    pub second: u16,
    pub third: u16,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTSkillStartNotifyInner {
    pub tripod_index: Option<TripodIndex>,
    pub tripod_level: Option<TripodLevel>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTSkillStartNotify {
    pub source_id: EntityId,
    pub skill_id: SkillId,
    pub skill_option_data: PKTSkillStartNotifyInner,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTSkillDamageAbnormalMoveNotifyInner {
    pub skill_damage_event: SkillDamageEvent,
    pub skill_move_option_data: SkillMoveOptionData
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTSkillDamageAbnormalMoveNotify {
    pub source_id: EntityId,
    pub skill_damage_abnormal_move_events: Vec<PKTSkillDamageAbnormalMoveNotifyInner>,
    pub skill_id: SkillId,
    pub skill_effect_id: SkillEffectId,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTSkillDamageNotify {
    pub source_id: EntityId,
    pub skill_damage_events: Vec<SkillDamageEvent>,
    pub skill_id: SkillId,
    pub skill_effect_id: Option<SkillEffectId>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTPartyInfoInner {
    pub name: String,
    pub class_id: ClassId,
    pub character_id: CharacterId,
    pub gear_level: GearLevel,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTPartyInfo {
    pub party_instance_id: PartyInstanceId,
    pub raid_instance_id: RaidInstanceId,
    pub party_member_datas: Vec<PKTPartyInfoInner>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTPartyLeaveResult {
    pub party_instance_id: PartyInstanceId,
    pub name: String
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTPartyStatusEffectAddNotify {
    pub character_id: u64,
    pub status_effect_datas: Vec<StatusEffectData>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTPartyStatusEffectRemoveNotify {
    pub character_id: CharacterId,
    pub status_effect_instance_ids: Vec<StatusEffectInstanceId>,
    pub reason: u8
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTPartyStatusEffectResultNotify {
    pub raid_instance_id: RaidInstanceId,
    pub party_instance_id: PartyInstanceId,
    pub character_id: CharacterId
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTStatusEffectAddNotify {
    pub object_id: EntityId,
    pub status_effect_data: StatusEffectData
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTStatusEffectRemoveNotify {
    pub object_id: EntityId,
    pub status_effect_instance_ids: Vec<StatusEffectInstanceId>,
    pub reason: u8
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTTriggerStartNotify {
    pub signal: u32,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTZoneMemberLoadStatusNotify {
    pub zone_id: u32,
    pub zone_level: u32
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTZoneObjectUnpublishNotify {
    pub object_id: u64
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTStatusEffectSyncDataNotify {
    pub object_id: EntityId,
    pub status_effect_instance_id: StatusEffectInstanceId,
    pub character_id: CharacterId,
    pub value: u64,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTTroopMemberUpdateMinNotify {
    pub character_id: u64,
    pub cur_hp: i64,
    pub max_hp: i64,
    pub status_effect_datas: Vec<StatusEffectData>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewTransit {
    pub channel_id: u32
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewPCInner {
    pub player_id: EntityId,
    pub name: String,
    pub class_id: ClassId,
    pub max_item_level: GearLevel,
    pub character_id: CharacterId,
    pub stat_pairs: Vec<StatPair>,
    pub equip_item_datas: Vec<EquipItemData>,
    pub status_effect_datas: Vec<StatusEffectData>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct PKTNewPC {
    pub pc_struct: PKTNewPCInner
}