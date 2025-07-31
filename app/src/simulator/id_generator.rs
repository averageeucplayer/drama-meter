use std::str::FromStr;

use hashbrown::HashSet;
use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};

use crate::{models::Class, simulator::utils::random_nickname};

#[derive(Debug)]
pub struct IdGenerator {
    used_u64: HashSet<u64>,
    used_u32: HashSet<u32>,
    rng: ThreadRng,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            used_u64: HashSet::new(),
            used_u32: HashSet::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn resolve_class(&mut self, class_id: &str) -> Class {
        match class_id {
            "<dps>" => {
                let class = Class::DPS().iter().choose(&mut self.rng).unwrap();
                *class
            },
            "<support>" => {
                let class = Class::SUPPORT().iter().choose(&mut self.rng).unwrap();
                *class
            },
            value => {
                let class = Class::from_str(value).unwrap();
                class
            }
        }
    }

    pub fn resolve_nickname(&mut self, name: &str) -> String {
        let name = match name {
            "<nickname>" => {
                random_nickname(&mut self.rng)
            }
            value => value.to_string()
        };

        name
    }

    pub fn resolve_u32(&mut self, template: &str) -> u32 {
        match template {
            "<u32>" => self.new_u32(),
            value => value.parse::<u32>().unwrap()
        }
    }

    pub fn new_u64(&mut self) -> u64 {
        let mut id: u64 = self.rng.random();

        while self.used_u64.contains(&id) {
            id = self.rng.random();
        }

        id
    }

    pub fn new_u32(&mut self) -> u32 {
        let mut id: u32 = self.rng.random();

        while self.used_u32.contains(&id) {
            id = self.rng.random();
        }

        id
    }
}