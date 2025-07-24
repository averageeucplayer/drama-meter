use std::rc::Weak;

use chrono::{Date, DateTime, Duration, Utc};
use hashbrown::HashMap;
use rand::{rngs::ThreadRng, Rng};

use crate::{models::{Class, HitFlag, HitOption}, simulator::{class_builder::*, skill_damage::SKILL_DAMAGE_MAP}};


pub trait SimulatorPlayer : std::fmt::Debug {
    fn id(&self) -> u64;
    fn character_id(&self) -> u64;
    fn adjust_skills(&mut self);
    fn buffs(&self) -> &HashMap<u32, SimulatorSkillBuff>;
    fn buffs_mut(&mut self) -> &mut HashMap<u32, SimulatorSkillBuff>;
    fn get_combat_stats(&self) -> SimulatorPlayerStats;
    fn get_available_skill(&mut self, now: DateTime<Utc>) -> Option<&mut SimulatorPlayerSkill>;
}

#[derive(Debug)]
pub struct BardPlayer {
    pub base: SimulatorPlayerBase,
}

impl SimulatorPlayer for BardPlayer {
    fn get_available_skill(&mut self, now: DateTime<Utc>) -> Option<&mut SimulatorPlayerSkill> {
        self.base.get_available_skill(now)
    }
    
    fn buffs(&self) -> &HashMap<u32, SimulatorSkillBuff> {
        &self.base.buffs
    }

    fn buffs_mut(&mut self) -> &mut HashMap<u32, SimulatorSkillBuff> {
        &mut self.base.buffs
    }
    
    fn adjust_skills(&mut self) {
        self.base.adjust_skills();
    }
    
    fn id(&self) -> u64 {
        self.base.id
    }
    
    fn character_id(&self) -> u64 {
        self.base.character_id
    }
    
    fn get_combat_stats(&self) -> SimulatorPlayerStats {
        let SimulatorPlayerBase {
            attack_power,
            crit_damage,
            crit_rate,
            cooldown_reduction,
            ..
        } = self.base;
        
        SimulatorPlayerStats {
            attack_power,
            cooldown_reduction,
            crit_damage,
            crit_rate
        }
    }
}

#[derive(Debug)]
pub struct BerserkerPlayer {
    pub base: SimulatorPlayerBase,
}

impl SimulatorPlayer for BerserkerPlayer {
    fn get_available_skill(&mut self, now: DateTime<Utc>) -> Option<&mut SimulatorPlayerSkill> {
        self.base.get_available_skill(now)
    }

    fn buffs(&self) -> &HashMap<u32, SimulatorSkillBuff> {
        &self.base.buffs
    }
    
    fn buffs_mut(&mut self) -> &mut HashMap<u32, SimulatorSkillBuff> {
        &mut self.base.buffs
    }
    
    fn adjust_skills(&mut self) {
        self.base.adjust_skills();
    }

    fn id(&self) -> u64 {
        self.base.id
    }
    
    fn character_id(&self) -> u64 {
        self.base.character_id
    }

    fn get_combat_stats(&self) -> SimulatorPlayerStats {
        let SimulatorPlayerBase {
            attack_power,
            crit_damage,
            crit_rate,
            cooldown_reduction,
            ..
        } = self.base;
        
        SimulatorPlayerStats {
            attack_power,
            cooldown_reduction,
            crit_damage,
            crit_rate
        }
    }
}

#[derive(Debug)]
pub struct PaladinPlayer {
    pub base: SimulatorPlayerBase,
}

impl SimulatorPlayer for PaladinPlayer {
    fn get_available_skill(&mut self, now: DateTime<Utc>) -> Option<&mut SimulatorPlayerSkill> {
        self.base.get_available_skill(now)
    }

    fn buffs(&self) -> &HashMap<u32, SimulatorSkillBuff> {
        &self.base.buffs
    }
    
    fn buffs_mut(&mut self) -> &mut HashMap<u32, SimulatorSkillBuff> {
        &mut self.base.buffs
    }
    
    fn adjust_skills(&mut self) {
        self.base.adjust_skills();
    }

    fn id(&self) -> u64 {
        self.base.id
    }
    
    fn character_id(&self) -> u64 {
        self.base.character_id
    }

    fn get_combat_stats(&self) -> SimulatorPlayerStats {
        let SimulatorPlayerBase {
            attack_power,
            crit_damage,
            crit_rate,
            cooldown_reduction,
            ..
        } = self.base;
        
        SimulatorPlayerStats {
            attack_power,
            cooldown_reduction,
            crit_damage,
            crit_rate
        }
    }
}

#[derive(Debug)]
pub struct ArtistPlayer {
    pub base: SimulatorPlayerBase,
}

impl SimulatorPlayer for ArtistPlayer {
    fn get_available_skill(&mut self, now: DateTime<Utc>) -> Option<&mut SimulatorPlayerSkill> {
        self.base.get_available_skill(now)
    }

    fn buffs(&self) -> &HashMap<u32, SimulatorSkillBuff> {
        &self.base.buffs
    }
    
    fn buffs_mut(&mut self) -> &mut HashMap<u32, SimulatorSkillBuff> {
        &mut self.base.buffs
    }
    
    fn adjust_skills(&mut self) {
        self.base.adjust_skills();
    }

    fn id(&self) -> u64 {
        self.base.id
    }
    
    fn character_id(&self) -> u64 {
        self.base.character_id
    }

    fn get_combat_stats(&self) -> SimulatorPlayerStats {
        let SimulatorPlayerBase {
            attack_power,
            crit_damage,
            crit_rate,
            cooldown_reduction,
            ..
        } = self.base;
        
        SimulatorPlayerStats {
            attack_power,
            cooldown_reduction,
            crit_damage,
            crit_rate
        }
    }
}

#[derive(Debug)]
pub struct SimulatorContext {
    pub boss_id: u64,
    pub boss_debuffs: HashMap<u32, SimulatorSkillBuff>,
    pub max_boss_hp: i64,
    pub current_boss_hp: i64,
}

#[derive(Debug)]
pub struct SimulatorParty {
    pub id: u32,
    pub members: Vec<Box<dyn SimulatorPlayer>>,
    pub buffs: HashMap<u32, SimulatorSkillBuff>,
}

#[derive(Debug, Clone)]
pub struct SimulatorPlayerStats {
    pub attack_power: u32,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub cooldown_reduction: f32
}

#[derive(Debug, Clone)]
pub struct SimulatorPlayerCreateArgs {
    pub id: u64,
    pub character_id: u64,
    pub class_id: Class,
    pub attack_power: u32,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub cooldown_reduction: f32
}

#[derive(Debug, Clone)]
pub struct SimulatorPlayerBase {
    pub rng: ThreadRng,
    pub id: u64,
    pub character_id: u64,
    pub class_id: Class,
    pub attack_power: u32,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub cooldown_reduction: f32,
    pub skills: Vec<SimulatorPlayerSkill>,
    pub identity_skill: SimulatorPlayerSkill,
    pub awakening_skill: SimulatorPlayerSkill,
    pub hyper_awakening_technique_skill: SimulatorPlayerSkill,
    pub hyper_awakening_skill: SimulatorPlayerSkill,
    pub buffs: HashMap<u32, SimulatorSkillBuff>,
}

impl SimulatorPlayerBase {
    
    pub fn get_available_skill(&mut self, now: DateTime<Utc>) -> Option<&mut SimulatorPlayerSkill> {
        
        let buffer = Duration::milliseconds(50);
        let skill = self.skills.iter_mut()
            .filter(|skill| skill.cooldown_ends_on < now + buffer )
            .max_by(|a, b| {
                a.priority
                    .cmp(&b.priority)
                    .then_with(|| b.cooldown_ends_on.cmp(&a.cooldown_ends_on))
            });

        skill
    }
}


#[derive(Debug, Default, Clone)]
pub struct SimulatorPlayerSkill {
    pub rng: ThreadRng,
    pub id: u32,
    pub priority: u8,
    pub deals_damage: bool,
    pub player_id: u64,
    pub min_damage_ratio: f64,
    pub max_damage_ratio: f64,
    pub buffs: Vec<SimulatorPlayerSkillBuff>,
    pub cooldown: Duration,
    pub cooldown_ends_on: DateTime<Utc>,
    pub tripod_index: Option<meter_core::packets::definitions::TripodIndex>,
    pub tripod_level: Option<meter_core::packets::definitions::TripodLevel>,
}

#[derive(Debug, Clone)]
pub struct SimulatorPlayerSkillConsumeResult {
    pub deals_damage: bool,
    pub tripod_index: Option<meter_core::packets::definitions::TripodIndex>,
    pub tripod_level: Option<meter_core::packets::definitions::TripodLevel>,
    pub source_id: u64,
    pub skill_id: u32,
    pub target_id: u64,
    pub damage: i64,
    pub current_boss_hp: i64,
    pub max_boss_hp: i64,
    pub hit_flag: HitFlag,
    pub hit_option: HitOption,
    pub buffs: Vec<SimulatorPlayerSkillBuff> 
}

#[derive(Debug, Clone, Copy)]
pub enum SimulatorPlayerSkillBuffCategory {
    Buff = 0,
    Debuff = 1
}

#[derive(Debug, Clone, Copy)]
pub enum SimulatorPlayerSkillBuffTarget {
    SelfTarget = 0,
    SelfParty = 1
}


#[derive(Debug, Clone)]
pub struct SimulatorPlayerSkillBuff {
    pub id: u32,
    pub duration: Duration,
    pub buff_type: SimulatorSkillBuffType,
    pub category: SimulatorPlayerSkillBuffCategory,
    pub target: SimulatorPlayerSkillBuffTarget
}

#[derive(Debug, Clone)]
pub struct SimulatorSkillBuff {
    pub id: u32,
    pub target_id: u64,
    pub expires_on: DateTime<Utc>,
    pub buff_type: SimulatorSkillBuffType,
    pub category: SimulatorPlayerSkillBuffCategory,
    pub target: SimulatorPlayerSkillBuffTarget
}

impl SimulatorSkillBuff {
    pub fn new(id: u32, target_id: u64, buff: SimulatorPlayerSkillBuff, expires_on: DateTime<Utc>) -> Self {
        let SimulatorPlayerSkillBuff {
            buff_type,
            category,
            target,
            ..
        } = buff;
        
        Self {
            id,
            target_id,
            expires_on,
            target,
            buff_type,
            category,
        }
    }

    // pub fn apply(&self, player: &mut SimulatorPlayer) {

    // }
}

#[derive(Debug, Clone)]
pub enum SimulatorSkillBuffType {
    Multiplicative(f32),
    ManaRegen,
    DamageReduction,
    Shield,
    Additive(f32),
}

impl SimulatorPlayerSkill {
    pub fn consume(
        &mut self,
        attack_power: u32,
        crit_rate: f64,
        crit_damage: f64,
        context: &mut SimulatorContext,
        now: DateTime<Utc>) -> SimulatorPlayerSkillConsumeResult {
        
        let mut hit_flag = HitFlag::Normal;
        let hit_option = HitOption::FlankAttack;
        let mut damage_f64 = 0.0;
        
        if self.deals_damage {
            let ratio = self.rng.random_range(self.min_damage_ratio..self.max_damage_ratio);
            damage_f64 = attack_power as f64 * ratio;
            let is_critical = self.rng.random_bool(crit_rate);

            if is_critical {
                damage_f64 = damage_f64 * crit_damage;
                hit_flag = HitFlag::Critical;
            }
        }


        self.cooldown_ends_on = now + self.cooldown;

        SimulatorPlayerSkillConsumeResult {
            deals_damage: self.deals_damage,
            skill_id: self.id,
            tripod_index: None,
            tripod_level: None,
            source_id: self.player_id,
            target_id: context.boss_id,
            current_boss_hp: context.current_boss_hp,
            max_boss_hp: context.max_boss_hp,
            damage: damage_f64 as i64,
            hit_flag,
            hit_option,
            buffs: self.buffs.clone()
        }
    }
}



impl SimulatorPlayerBase {
    pub fn new(args: SimulatorPlayerCreateArgs) -> Box<dyn SimulatorPlayer> {
        let mut player = match args.class_id {
            Class::Bard => bard(args),
            Class::Paladin => paladin(args),
            Class::Artist => artist(args),
            Class::Berserker => berserker(args),
            _ => berserker(args)
        };

        player.adjust_skills();

        player
    }

    pub fn adjust_skills(&mut self) {
        for skill in self.skills.iter_mut() {
            let cooldown = skill.cooldown.num_milliseconds() as f32 * (1.0 - self.cooldown_reduction);
            skill.cooldown = Duration::milliseconds(cooldown as i64);

            let (min, max) = SKILL_DAMAGE_MAP.get(&skill.id).unwrap_or_else(|| &(1.0, 2.0));
            skill.min_damage_ratio = *min;
            skill.max_damage_ratio = *max;
        }
    }
}
