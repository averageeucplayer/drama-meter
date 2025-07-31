use std::{fs, path::{Path, PathBuf}};

use rand::{distr::{Alphabetic, Alphanumeric, SampleString}, rngs::ThreadRng, seq::IteratorRandom};
use anyhow::*;
use crate::models::{HitFlag, HitOption};

pub fn parse_range(s: &str) -> Option<(f32, f32)> {
    let trimmed = s.trim_matches(&['<', '>'][..]);
    let parts: Vec<&str> = trimmed.split("..").collect();
    
    if parts.len() == 2 {
        let start: u32 = parts[0].parse().ok()?;
        let end: u32 = parts[1].parse().ok()?;
        Some((start as f32, end as f32))
    } else {
        None
    }
}

pub fn random_nickname(rng: &mut ThreadRng) -> String {
    let mut string = Alphabetic.sample_string(rng, 10);

    let char = string.get_mut(0..1).unwrap();
    char.make_ascii_uppercase();

    let str = string.get_mut(1..).unwrap();
    str.make_ascii_lowercase();

    string
}

pub fn encode_modifier(hit_flag: HitFlag, hit_option: HitOption) -> i32 {
    let flag_bits = if hit_flag == HitFlag::Unknown {
        15u8
    } else {
        hit_flag as u8
    };

    let option_bits = if (hit_option as u8) >= 4 {
        0u8
    } else {
        hit_option as u8
    };

    let value = (option_bits << 4) | flag_bits;

    value as i32
}

