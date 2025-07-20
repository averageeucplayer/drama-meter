use std::rc::Rc;

#[derive(Debug)]
pub struct Npc {
    pub name: String,
    pub npc_id: u32,
    pub push_immune: bool,
    pub level: u16,
    pub balance_level: u16,
}

#[derive(Debug, Default)]
pub struct Esther {
    pub npc_id: u32,
    pub name: String,
    pub damage_dealt: i64,
}

#[derive(Debug, Default)]
pub struct Boss {
    pub id: u64,
    pub name: Rc<String>,
    pub npc_id: u32,
    pub push_immune: bool,
    pub level: u16,
    pub balance_level: u16,
    pub current_shield: u64,
    pub encounter_stats: BossStats
}

#[derive(Debug, Default)]
pub struct BossStats {
    pub dealt: i64,
    pub taken: i64,
    pub current_shield: u64,
    pub current_hp: i64,
    pub max_hp: i64,
}