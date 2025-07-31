mod models;
mod utils;
mod packet;
mod id_generator;

use std::{fs::{self, File}, path::{Path, PathBuf}, str::FromStr, sync::mpsc, thread::sleep, time::Duration, vec};
use anyhow::*;
use bincode::{config, decode_from_slice, encode_to_vec};
use chrono::{DateTime, Utc};
use hashbrown::{hash_map::Entry, HashMap, HashSet};
use meter_core::packets::{definitions::*, opcodes::Pkt, structures::*};
use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};
use serde::{Deserialize, Serialize};

use crate::{misc::data::NPC_DATA, models::*, simulator::{id_generator::IdGenerator, models::*, packet::*, utils::*}};

static CONFIG: config::Configuration = config::standard();

pub type Packet = (Pkt, Vec<u8>);

pub struct Simulator {
    id_generator: IdGenerator,
    rng: ThreadRng,
    data: EncounterTemplate,
    ids: HashSet<u64>,
    instance_ids: HashSet<u32>,
    character_ids: HashSet<u64>,
}


impl Simulator {
    pub fn new(path: &Path) -> Result<Self> {
        let mut rng = rand::rng();
        let file = File::open(path)?;
        let data: EncounterTemplate = serde_json::from_reader(file)?;
        let id_generator = IdGenerator::new();

        Ok(Self {
            id_generator,
            rng,
            data,
            ids: HashSet::new(),
            instance_ids: HashSet::new(),
            character_ids: HashSet::new(),
        })
    }

    pub fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn tick(&mut self, now: DateTime<Utc>) -> Option<Vec<Packet>> {
        None
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_load_encounter_template() {
        let path = PathBuf::from(r#"C:\repos\drama-meter\app\assets\templates\mordum_g3.json"#);
        let mut simulator = Simulator::new(&path).unwrap();

        simulator.setup().unwrap();

        loop {
            let now = Utc::now();
            let packets = simulator.tick(now);

            for packet in packets {
                
            }
        }
    }
}