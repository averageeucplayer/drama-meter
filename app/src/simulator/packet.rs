use bincode::encode_to_vec;
use meter_core::packets::{definitions::*, opcodes::Pkt, structures::{NpcStruct, NpcStructBalance, SkillDamageEvent, StatPair, StatusEffectData}};

use crate::{models::{HitFlag, HitOption}, simulator::{utils::encode_modifier, Packet, CONFIG}};

pub fn encode_skill_damage_packet(
    source_id: u64,
    skill_id: u32,
    target_id: u64,
    hit_flag: HitFlag,
    hit_option: HitOption,
    current_boss_hp: i64,
    max_boss_hp: i64,
    damage: i64,
) -> Packet {
     let packet = PKTSkillDamageNotify {
        source_id,
        skill_id,
        skill_damage_events: vec![SkillDamageEvent {
            target_id,
            modifier: encode_modifier(hit_flag, hit_option),
            cur_hp: current_boss_hp,
            max_hp: max_boss_hp,
            damage: damage,
            ..Default::default()
        }],
        skill_effect_id: None,
    };

    let bytes = encode_to_vec(packet, CONFIG).unwrap();
    let packet = (Pkt::SkillDamageNotify, bytes);

    packet
}

pub fn encode_skill_cast_notify(source_id: u64, skill_id: u32) -> Packet {
    let pkt = PKTSkillCastNotify { source_id, skill_id };
    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::SkillCastNotify, bytes)
}

pub fn encode_skill_start_notify(
    source_id: u64,
    skill_id: u32,
    tripod_index: Option<meter_core::packets::definitions::TripodIndex>,
    tripod_level: Option<meter_core::packets::definitions::TripodLevel>) -> Packet {
    let pkt = PKTSkillStartNotify {
        skill_id,
        skill_option_data: PKTSkillStartNotifyInner {
            tripod_index,
            tripod_level,
        },
        source_id,
    };
    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::SkillStartNotify, bytes)
}

pub fn encode_status_effect_add_notify(
    object_id: u64,
    source_id: u64,
    status_effect_id: u32,
    status_effect_instance_id: u32,
    total_time: f32
) -> Packet {
    let status_effect_data = StatusEffectData {
        source_id,
        status_effect_id,
        status_effect_instance_id,
        total_time,
        ..Default::default()
    };
    let pkt = PKTStatusEffectAddNotify {
        object_id,
        status_effect_data,
    };
    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::StatusEffectAddNotify, bytes)
}

pub fn encode_party_status_effect_add_notify(
    character_id: u64,
    effects: Vec<(
        u64, // source_id
        u32, // status_effect_id
        u32, // status_effect_instance_id
        f32, // total_time
    )>,
) -> Packet {
    let status_effect_datas: Vec<StatusEffectData> = effects
        .into_iter()
        .map(|(source_id, status_effect_id, status_effect_instance_id, total_time)| {
            StatusEffectData {
                source_id,
                status_effect_id,
                status_effect_instance_id,
                total_time,
                ..Default::default()
            }
        })
        .collect();

    let pkt = PKTPartyStatusEffectAddNotify {
        character_id,
        status_effect_datas,
    };

    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::PartyStatusEffectAddNotify, bytes)
}

pub fn encode_party_info(
    party_instance_id: u32,
    raid_instance_id: u32,
    party_member_datas: Vec<PKTPartyInfoInner>,
) -> Packet {
    let pkt = PKTPartyInfo {
        party_instance_id,
        raid_instance_id,
        party_member_datas,
    };
    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::PartyInfo, bytes)
}

pub fn encode_init_pc(
    player_id: u64,
    name: String,
    class_id: u32,
    gear_level: f32,
    character_id: u64,
    hp: i64
) -> Packet {
    let pkt = PKTInitPC {
        player_id,
        name,
        class_id,
        gear_level,
        character_id,
        stat_pairs: vec![
            StatPair { stat_type: 1, value: hp as i64 },
            StatPair { stat_type: 27, value: hp as i64 }
        ],
        status_effect_datas: vec![],
    };
    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::InitPC, bytes)
}

    // let data = PKTNewPC {
                //     pc_struct: PKTNewPCInner {
                //         player_id,
                //         name,
                //         class_id: class_id as u32,
                //         max_item_level: gear_level,
                //         character_id,
                //         stat_pairs: vec![
                //             StatPair { stat_type: 1, value: hp as i64 },
                //             StatPair { stat_type: 27, value: hp as i64 }
                //         ],
                //         equip_item_datas: vec![],
                //         status_effect_datas: vec![]
                //     }
                // };

                // let data = bincode::encode_to_vec(data, CONFIG)?;
                // let packet = (Pkt::NewPC, data);

pub fn encode_new_pc(
    player_id: u64,
    name: String,
    class_id: u32,
    gear_level: f32,
    character_id: u64,
    hp: i64
) -> Packet {
    let pkt = PKTNewPC {
        pc_struct: PKTNewPCInner {
            player_id,
            name,
            class_id,
            max_item_level: gear_level,
            character_id,
            stat_pairs: vec![
                StatPair { stat_type: 1, value: hp as i64 },
                StatPair { stat_type: 27, value: hp as i64 }
            ],
            equip_item_datas: vec![],
            status_effect_datas: vec![],
        },
    };
    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::NewPC, bytes)
}

pub fn encode_new_npc(
    object_id: u64,
    type_id: u32,
    level: u16,
    balance_level: Option<u16>,
    hp: i64
) -> Packet {
    let npc_struct = NpcStruct {
        object_id,
        type_id,
        level,
        balance_level: NpcStructBalance { value: balance_level },
        stat_pairs: vec![
            StatPair { stat_type: 1, value: hp as i64 },
            StatPair { stat_type: 27, value: hp as i64 }
        ],
        status_effect_datas: vec![],
    };
    let pkt = PKTNewNpc { npc_struct };
    let bytes = encode_to_vec(pkt, CONFIG).unwrap();
    (Pkt::NewNpc, bytes)
}

pub fn encode_remove_object(object_id: u64) -> Packet {
    let packet = PKTZoneObjectUnpublishNotify { object_id };
    let bytes = encode_to_vec(packet, CONFIG).unwrap();
    (Pkt::NewNpcSummon, bytes)
}

pub fn encode_new_npc_summon(
    object_id: u64,
    owner_id: u64,
    type_id: u32,
    level: u16,
    balance_level: Option<u16>,
    hp: i64
) -> Packet {
    let npc_struct = NpcStruct {
        object_id,
        type_id,
        level,
        balance_level: NpcStructBalance { value: balance_level },
        stat_pairs: vec![
            StatPair { stat_type: 1, value: hp as i64 },
            StatPair { stat_type: 27, value: hp as i64 }
        ],
        status_effect_datas: vec![],
    };
    let packet = PKTNewNpcSummon { owner_id, npc_struct };
    let bytes = encode_to_vec(packet, CONFIG).unwrap();
    (Pkt::NewNpcSummon, bytes)
}

pub fn encode_party_status_effect_remove_notify(character_id: u64, reason: u8, id: u32) -> Packet {
    let packet = PKTPartyStatusEffectRemoveNotify {
        character_id,
        reason,
        status_effect_instance_ids: vec![id]
    };

    let bytes = encode_to_vec(packet, CONFIG).unwrap();
    (Pkt::PartyStatusEffectRemoveNotify, bytes)
}

pub fn encode_status_effect_remove_notify(object_id: u64, reason: u8, id: u32) -> Packet {
    let packet = PKTStatusEffectRemoveNotify {
        object_id,
        reason,
        status_effect_instance_ids: vec![id]
    };

    let bytes = encode_to_vec(packet, CONFIG).unwrap();
    (Pkt::StatusEffectRemoveNotify, bytes)
}