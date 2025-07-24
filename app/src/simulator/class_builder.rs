use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use rand::rngs::ThreadRng;

use crate::{models::Class, simulator::{enums::{ArtistSkills, BardSkillBuffs, BardSkills, BerserkerBuffSkills, BerserkerSkills, PaladinSkills}, player::*}};


pub fn bard(args: SimulatorPlayerCreateArgs) -> Box<dyn SimulatorPlayer> {

    let SimulatorPlayerCreateArgs {
        attack_power,
        cooldown_reduction,
        crit_rate,
        crit_damage,
        character_id,
        class_id,
        id
    } = args;

    let mut skills = vec![];
    let rng = rand::rng();

    let awakening_skill = SimulatorPlayerSkill {
        id: BardSkills::Symphonia as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(300),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let hyper_awakening_skill = SimulatorPlayerSkill {
        id: BardSkills::SymphonyMelody as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(300),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let identity_skill = SimulatorPlayerSkill {
        id: BardSkills::SerenadeOfCourage15 as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(1),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::SerenadeOfCourage15 as u32,
                buff_type: SimulatorSkillBuffType::Multiplicative(1.15),
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(15)
            }
        ],
        rng: rng.clone(),
        ..Default::default()
    };

    let hyper_awakening_technique_skill = SimulatorPlayerSkill {
        id: BardSkills::Aria as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(90),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::AriaHyperAwakeningSkillDamage as u32,
                buff_type: SimulatorSkillBuffType::Multiplicative(1.1),
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(7)
            },
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::AriaOutgoingDamage as u32,
                buff_type: SimulatorSkillBuffType::Multiplicative(1.1),
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(7)
            }
        ],
        rng: rng.clone(),
        ..Default::default()
    };

    let mut player = BardPlayer { 
        base: SimulatorPlayerBase {
            id,
            character_id,
            class_id: Class::Bard,
            attack_power,
            crit_rate,
            crit_damage,
            cooldown_reduction,
            skills: vec![],
            awakening_skill,
            hyper_awakening_skill,
            hyper_awakening_technique_skill,
            identity_skill,
            buffs: HashMap::new(),
            rng: rng.clone(),
        }
    };

    let skill = SimulatorPlayerSkill {
        id: BardSkills::Sonatina as u32,
        priority: 4,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::SonatinaNoteBrand as u32,
                buff_type: SimulatorSkillBuffType::Multiplicative(1.1),
                category: SimulatorPlayerSkillBuffCategory::Debuff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(7)
            }
        ],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BardSkills::WindOfMusic as u32,
        priority: 3,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(18),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BardSkills::PreludeOfStorm as u32,
        priority: 2,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(16),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BardSkills::SonicVibration as u32,
        priority: 1,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(24),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::SonicVibrationManaRegen as u32,
                buff_type: SimulatorSkillBuffType::ManaRegen,
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(7)
            },
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::SonicVibrationAtkPower as u32,
                buff_type: SimulatorSkillBuffType::Additive(attack_power as f32 * 0.15),
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(4)
            }
        ],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BardSkills::HeavenlyTune as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(30),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::HeavenlyTuneManaRegen as u32,
                buff_type: SimulatorSkillBuffType::ManaRegen,
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(7)
            },
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::IntenseTune as u32,
                buff_type: SimulatorSkillBuffType::Additive(attack_power as f32 * 0.15),
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(7)
            }
        ],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

        let skill = SimulatorPlayerSkill {
        id: BardSkills::GuardianTune as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(30),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::GuardianTuneDamageShield as u32,
                buff_type: SimulatorSkillBuffType::DamageReduction,
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(7)
            },
            SimulatorPlayerSkillBuff {
                id: BardSkillBuffs::GuardianTuneDamageReduction as u32,
                buff_type: SimulatorSkillBuffType::Shield,
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfParty,
                duration: Duration::seconds(4)
            }
        ],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    player.base.skills = skills;

    Box::new(player)
}

pub fn paladin(args: SimulatorPlayerCreateArgs) -> Box<dyn SimulatorPlayer> {

    let SimulatorPlayerCreateArgs {
        attack_power,
        cooldown_reduction,
        crit_rate,
        crit_damage,
        character_id,
        class_id,
        id
    } = args;

    let mut skills = vec![];
    let rng = rand::rng();

     let awakening_skill = SimulatorPlayerSkill {
        id: PaladinSkills::AlithanesJudgment as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let hyper_awakening_skill = SimulatorPlayerSkill {
        id: PaladinSkills::AlithanesDevotion as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let hyper_awakening_technique_skill = SimulatorPlayerSkill {
        id: PaladinSkills::DivineJustice as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(90),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let identity_skill = SimulatorPlayerSkill {
        id: PaladinSkills::HolyAura as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let mut player = PaladinPlayer { 
        base: SimulatorPlayerBase {
            id, 
            character_id,
            class_id,
            attack_power,
            cooldown_reduction,
            crit_rate,
            crit_damage,
            skills: vec![],
            awakening_skill,
            hyper_awakening_skill,
            hyper_awakening_technique_skill,
            identity_skill,
            buffs: HashMap::new(),
            rng: rng.clone()   
        }
    };

    player.base.skills = skills;

    Box::new(player)
}

pub fn artist(args: SimulatorPlayerCreateArgs) -> Box<dyn SimulatorPlayer> {

    let SimulatorPlayerCreateArgs {
        attack_power,
        cooldown_reduction,
        crit_rate,
        crit_damage,
        character_id,
        class_id,
        id
    } = args;

    let mut skills = vec![];
    let rng = rand::rng();

    let awakening_skill = SimulatorPlayerSkill {
        id: ArtistSkills::MasterworkEfflorescence as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let hyper_awakening_skill = SimulatorPlayerSkill {
        id: ArtistSkills::DreamBlossomGarden as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

     let hyper_awakening_technique_skill = SimulatorPlayerSkill {
        id: ArtistSkills::PaintDragonEngraving as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(90),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

     let identity_skill = SimulatorPlayerSkill {
        id: ArtistSkills::Moonfall as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let mut player = ArtistPlayer { 
        base: SimulatorPlayerBase {
            id, 
            character_id,
            class_id,
            attack_power,
            cooldown_reduction,
            crit_rate,
            crit_damage,
            skills: vec![],
            awakening_skill,
            hyper_awakening_skill,
            hyper_awakening_technique_skill,
            identity_skill,
            buffs: HashMap::new(),
            rng: rng.clone()
        }
    };

    let skill = SimulatorPlayerSkill {
        id: ArtistSkills::StrokeHopper as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: ArtistSkills::PaintSunWell as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(30),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: ArtistSkills::PaintSunsketch as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(30),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    player.base.skills = skills;

    Box::new(player)
}


pub fn berserker(args: SimulatorPlayerCreateArgs) -> Box<dyn SimulatorPlayer> {

    let SimulatorPlayerCreateArgs {
        attack_power,
        cooldown_reduction,
        crit_rate,
        crit_damage,
        character_id,
        class_id,
        id
    } = args;

    let mut skills = vec![];
    let rng = rand::rng();
   
    let awakening_skill = SimulatorPlayerSkill {
        id: BerserkerSkills::BerserkFury as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let hyper_awakening_skill = SimulatorPlayerSkill {
        id: BerserkerSkills::RageDeathblade as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let identity_skill = SimulatorPlayerSkill {
        id: BerserkerSkills::BloodyRush as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let hyper_awakening_technique_skill = SimulatorPlayerSkill {
        id: BerserkerSkills::BloodSlash as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    let mut player = BerserkerPlayer { 
        base: SimulatorPlayerBase {
            id,
            character_id,
            class_id: Class::Berserker,
            attack_power,
            crit_rate,
            crit_damage,
            cooldown_reduction,
            skills: vec![],
            awakening_skill,
            hyper_awakening_skill,
            hyper_awakening_technique_skill,
            identity_skill,
            buffs: HashMap::new(),
            rng: rng.clone(),
        }
    };

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::RedDust as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![
            SimulatorPlayerSkillBuff {
                id: BerserkerBuffSkills::RedDustAtkPower as u32,
                buff_type: SimulatorSkillBuffType::Multiplicative(1.1),
                category: SimulatorPlayerSkillBuffCategory::Buff,
                target: SimulatorPlayerSkillBuffTarget::SelfTarget,
                duration: Duration::seconds(8)
            }
        ],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::HellBlade as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::Overdrive as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::FinishStrike as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::SwordStorm as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::MountainCrash as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::FinishStrike as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::PowerBreak as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    let skill = SimulatorPlayerSkill {
        id: BerserkerSkills::TempestSlash as u32,
        priority: 0,
        deals_damage: true,
        player_id: id,
        cooldown: Duration::seconds(5),
        cooldown_ends_on: DateTime::<Utc>::MIN_UTC,
        buffs: vec![],
        rng: rng.clone(),
        ..Default::default()
    };

    skills.push(skill);

    player.base.skills = skills;

    Box::new(player)
}