use std::{fs, path::{Path, PathBuf}};

use rand::{distr::{Alphanumeric, SampleString}, rngs::ThreadRng, seq::IteratorRandom};
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
    let mut string = Alphanumeric.sample_string(rng, 10);

    let char = string.get_mut(0..1).unwrap();
    char.make_ascii_uppercase();

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

pub fn get_templates(templates_path: &Path) -> Vec<PathBuf> {
    let template_files: Vec<_> = fs::read_dir(&templates_path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file()
                && entry
                    .path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    template_files
}

pub fn get_random_template(templates: &mut Vec<PathBuf>) -> Option<PathBuf> {

    let mut rng = rand::rng();
    let idx = (0..templates.len()).choose(&mut rng)?;
    Some(templates.remove(idx))
}

pub fn move_to_processed(template_path: &Path, destination_path: &Path) -> Result<()> {
    fs::rename(template_path, destination_path)?;
    Ok(())
}
