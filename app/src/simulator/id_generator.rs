use hashbrown::HashSet;
use rand::{rngs::ThreadRng, Rng};

use crate::simulator::utils::random_nickname;

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