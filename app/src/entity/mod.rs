pub mod player;
pub mod npc;

use chrono::{DateTime, Utc};
use hashbrown::HashMap;
use meter_core::packets::structures::StatPair;
use std::{fmt, ops::DerefMut};

use crate::{misc::data::{ESTHER_BY_NPC_ID, NPC_DATA}, entity::{npc::{Boss, Esther, Npc}, player::Player}};
use std::ops::Deref;

#[derive(Debug)]
pub struct Entity(BaseEntity, EntityVariant);

#[derive(Debug)]
pub enum EntityVariant {
    Unknown,
    Projectile(Projectile),
    Player(Player),
    Npc(Npc),
    Boss(Boss),
    Esther(Esther),
}

impl Entity {

    fn is_boss(npc_name: &str, max_hp: i64, grade: &str) -> bool {
        (grade == "boss"
            || grade == "raid"
            || grade == "epic_raid"
            || grade == "commander")
            && max_hp > 10_000
            && !npc_name.is_empty()
            && !npc_name.contains('_')
            && npc_name.is_ascii()
    }

    pub fn npc(
        id: u64,
        npc_id: u32,
        owner_id: Option<u64>,
        level: u16,
        balance_level: u16,
        max_hp: i64,
        stat_pairs: Vec<StatPair>,
        created_on: DateTime<Utc> ) -> Self {
        let base = BaseEntity { id, owner_id, created_on };

        if let Some(esther) = ESTHER_BY_NPC_ID.get(&npc_id) {
            let entity = Esther {
                npc_id,
                name: esther.name.clone(),
                ..Default::default()
            };
            let entity = EntityVariant::Esther(entity);
            return Self(base, entity)
        }

        if let Some(npc_info) = NPC_DATA.get(&npc_id) {
            let name = npc_info.name.clone().unwrap_or_default();

            if Self::is_boss(&name, max_hp, &npc_info.grade) {
                let entity = Boss { 
                    id, 
                    name: name.into(),
                    npc_id,
                    push_immune: true,
                    level,
                    balance_level,
                    current_shield: 0,
                    encounter_stats: npc::BossStats {
                        current_hp: max_hp,
                        max_hp: max_hp,
                        ..Default::default()
                    }
                };

                let entity = EntityVariant::Boss(entity);
                return Self(base, entity)
            }

            let entity = Npc {
                name,
                npc_id,
                push_immune: false,
                level,
                balance_level,
            };
            
            let entity = EntityVariant::Npc(entity);
            return Self(base, entity)
        }

        let entity = Npc {
            name: format!("{:x}", id),
            npc_id,
            push_immune: false,
            level,
            balance_level,
        };
        
        let entity = EntityVariant::Npc(entity);
        Self(base, entity)
    }

    pub fn trap(
        id: u64,
        owner_id: u64,
        skill_id: u32,
        skill_effect_id: u32,
        created_on: DateTime<Utc>
    ) -> Self {
        let base = BaseEntity { id, owner_id: Some(owner_id), created_on };
        let entity = Projectile {
            skill_id,
            is_attack_battle_item: false,
            skill_effect_id
        };
        let entity = EntityVariant::Projectile(entity);
        Self(base, entity)
    }

    pub fn projectile(
        id: u64,
        owner_id: u64,
        is_attack_battle_item: bool,
        skill_id: u32,
        skill_effect_id: u32,
        created_on: DateTime<Utc>
    ) -> Self {
        let base = BaseEntity { id, owner_id: Some(owner_id), created_on };
        let entity = Projectile {
            skill_id,
            is_attack_battle_item,
            skill_effect_id
        };
        let entity = EntityVariant::Projectile(entity);
        Self(base, entity)
    }

    pub fn unknown_player(
        id: u64,
        class_id: u32,
        created_on: DateTime<Utc>) -> Self {
        let base = BaseEntity { id, owner_id: None, created_on };
        let player = Player {
            id, 
            is_local: false,
            name: "".into(),
            class_id,
            gear_level: 0.0,
            character_id: 0,
            game_stats: HashMap::new(),
            encounter_stats: Default::default(),
            incapacitations: vec![]
        };
        let entity = EntityVariant::Player(player);
        Self(base, entity)
    }

    pub fn player(
        id: u64,
        player: Player,
        created_on: DateTime<Utc>) -> Self {
        let base = BaseEntity { id, owner_id: None, created_on };
        let entity = EntityVariant::Player(player);
        Self(base, entity)
    }

    pub fn unknown(id: u64, created_on: DateTime<Utc>) -> Self {
        let base = BaseEntity { id, owner_id: None, created_on };
        let entity = EntityVariant::Unknown;
        Self(base, entity)
    }

    pub fn unknown_local(id: u64, created_on: DateTime<Utc>) -> Self {
        let base = BaseEntity { id, owner_id: None,created_on };
        let mut player = Player::default();
        player.is_local = true;
        let entity = EntityVariant::Player(player);
        Self(base, entity)
    }

    pub fn character_id(&self) -> Option<u64> {
        match &self.1 {
            EntityVariant::Player(player) => Some(player.character_id),
            _ => None
        }
    }

    pub fn id(&self) -> u64 {
        self.0.id
    }

    pub fn get_owner(&self) -> Option<u64> {
        self.0.owner_id
    }

    pub fn as_boss(&self) -> Option<&Boss> {
        match &self.1 {
            EntityVariant::Boss(boss) => Some(boss),
            _ => None
        }
    }

    pub fn as_boss_mut(&mut self) -> Option<&mut Boss> {
        match &mut self.1 {
            EntityVariant::Boss(boss) => Some(boss),
            _ => None
        }
    }

    pub fn as_player(&self) -> Option<&Player> {
        match &self.1 {
            EntityVariant::Player(player) => Some(player),
            _ => None
        }
    }

    pub fn as_player_mut(&mut self) -> Option<&mut Player> {
        match &mut self.1 {
            EntityVariant::Player(player) => Some(player),
            _ => None
        }
    }
}

impl Deref for Entity {
    type Target = EntityVariant;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl DerefMut for Entity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

#[derive(Debug)]
pub struct BaseEntity {
    id: u64,
    owner_id: Option<u64>,
    created_on: DateTime<Utc>
}

#[derive(Debug)]
pub struct Projectile {
    pub is_attack_battle_item: bool,
    pub skill_effect_id: u32,
    pub skill_id: u32,
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = &self.0;
        match &self.1 {
            EntityVariant::Unknown => {
                write!(f, "Unknown Entity(id={}, created_on={})", base.id, base.created_on)
            }
            EntityVariant::Projectile(proj) => {
                write!(
                    f,
                    "Projectile(id={}, skill_id={}, skill_effect_id={}, attack_battle_item={}, created_on={})",
                    base.id, proj.skill_id, proj.skill_effect_id, proj.is_attack_battle_item, base.created_on
                )
            }
            EntityVariant::Player(player) => {
                write!(
                    f,
                    "Player(id={}, name='{}', class_id={}, created_on={})",
                    base.id, player.name, player.class_id, base.created_on
                )
            }
            EntityVariant::Npc(npc) => {
                write!(
                    f,
                    "Npc(id={}, name='{}', npc_id={}, level={}, created_on={})",
                    base.id, npc.name, npc.npc_id, npc.level, base.created_on
                )
            }
            EntityVariant::Boss(boss) => {
                write!(
                    f,
                    "Boss(id={}, name='{}', npc_id={}, level={}, created_on={})",
                    base.id, boss.name, boss.npc_id, boss.level, base.created_on
                )
            }
            EntityVariant::Esther(esther) => {
                write!(
                    f,
                    "Esther(id={}, name='{}', npc_id={}, created_on={})",
                    base.id, esther.name, esther.npc_id, base.created_on
                )
            }
        }
    }
}