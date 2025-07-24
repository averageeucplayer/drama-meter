use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::simulator::enums::{BardSkills, BerserkerSkills};

pub static SKILL_DAMAGE_MAP: Lazy<HashMap<u32, (f64, f64)>> = Lazy::new(|| {
    [
        (BardSkills::Sonatina as u32, (1.0, 2.0)),
        (BardSkills::SonicVibration as u32, (1.0, 2.0)),
        (BardSkills::HeavenlyTune as u32, (1.0, 2.0)),
        (BardSkills::Aria as u32, (10.0, 20.0)),
        (BardSkills::Symphonia as u32, (10.0, 20.0)),
        (BardSkills::SymphonyMelody as u32, (1000.0, 2000.0)),
        (BerserkerSkills::RedDust as u32, (1.0, 2.0)),
        (BerserkerSkills::AssaultBlade as u32, (1.0, 2.0)),
        (BerserkerSkills::BloodSlash as u32, (1.0, 2.0)),
        (BerserkerSkills::BloodySurge as u32, (1.0, 2.0)),
        (BerserkerSkills::BraveSlash as u32, (1.0, 2.0)),
        (BerserkerSkills::FinishStrike as u32, (1.0, 2.0)),
        (BerserkerSkills::FuryMethod as u32, (1.0, 2.0)),
    ].iter().cloned().collect()
});