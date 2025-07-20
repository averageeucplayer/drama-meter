use serde::{Deserialize, Serialize};
use bincode::{Decode, Encode};
use crate::types::*;

#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct StatusEffectDataValue {
    pub bytearray_0: Option<Vec<u8>>,
}

#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct StatusEffectData {
    
    pub source_id: EntityId,
    pub status_effect_id: StatusEffectId,
    pub status_effect_instance_id: StatusEffectInstanceId,
    pub value: StatusEffectDataValue,
    pub total_time: f32,
    pub stack_count: u8,
    pub end_tick: u64
}

#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct EquipItemData {

}

#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct NpcStructBalance {
    pub value: Option<u16>
}

#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct NpcStruct {
    pub object_id: EntityId,
    pub type_id: NpcId,
    pub level: u16,
    pub balance_level: NpcStructBalance,
    pub stat_pairs: Vec<StatPair>,
    pub status_effect_datas: Vec<StatusEffectData>
}

#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct StatPair {
    pub stat_type: u8,
    pub value: i64
}

#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct SkillDamageEventInner {
    pub p64_0: Option<i64>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct SkillDamageEventShield {
    pub p64_0: Option<i64>
}


#[derive(Debug, Default, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct SkillDamageEvent {
    pub target_id: u64,
    pub damage: i64,
    pub modifier: i32,
    pub cur_hp: i64,
    pub max_hp: i64,
    pub damage_attr: Option<u8>,
    pub damage_type: u8,
    pub shield_damage: SkillDamageEventShield,
}