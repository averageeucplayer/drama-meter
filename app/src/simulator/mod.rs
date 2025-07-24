mod models;
mod utils;
mod player;
mod enums;
mod class_builder;
mod skill_damage;
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

use crate::{misc::data::NPC_DATA, models::*, simulator::{id_generator::IdGenerator, models::*, packet::*, player::*, utils::*}};

static CONFIG: config::Configuration = config::standard();


pub fn process(template_path: PathBuf, destination_dir: PathBuf) {
    let (tx, rx) = mpsc::channel::<(Pkt, Vec<u8>)>();
    let mut templates = get_templates(&template_path);
    let next_encounter_delay = Duration::from_secs(10);

    loop {
        let path = get_random_template(&mut templates);

        let path = match path {
            Some(path) => path,
            None => {
                templates = get_templates(&template_path);
                sleep(next_encounter_delay);
                continue;
            },
        };

        let mut simulator = Simulator::new(&path).unwrap();

        simulator.setup();

        let mut now = Utc::now();

        while let Some(packets) = simulator.tick(now) {
            for packet in packets {
                tx.send(packet).unwrap();
            }
            now = Utc::now();
        }

        let file_name = path.file_name().unwrap();
        let destination_path = destination_dir.join(file_name);
        move_to_processed(&path, &destination_path);

        sleep(next_encounter_delay)
    }
}

pub type Packet = (Pkt, Vec<u8>);

pub struct Simulator {
    id_generator: IdGenerator,
    rng: ThreadRng,
    context: SimulatorContext,
    data: EncounterTemplate,
    packets: Vec<Packet>,
    ids: HashSet<u64>,
    instance_ids: HashSet<u32>,
    character_ids: HashSet<u64>,
    parties: Vec<SimulatorParty>,
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
            context: SimulatorContext {
                boss_debuffs: HashMap::new(),
                boss_id: 0,
                current_boss_hp: 0,
                max_boss_hp: 0,
            },
            packets: vec![],
            ids: HashSet::new(),
            instance_ids: HashSet::new(),
            character_ids: HashSet::new(),
            parties: vec![],
        })
    }

    pub fn setup(&mut self) -> Result<()> {
        self.context.boss_id = self.id_generator.new_u64();
        self.context.max_boss_hp = self.data.boss.hp as i64;
        self.context.current_boss_hp = self.context.max_boss_hp;
        let mut rng = rand::rng();

        let packet = encode_new_npc(
            self.context.boss_id,
            self.data.boss.id,
            self.data.boss.level,
            Some(self.data.boss.level),
            self.context.max_boss_hp);
        self.packets.push(packet);

        let raid_id = match self.data.raid.id.as_str() {
              "<u32>" => {
                self.rng.random()
            }
            value => self.data.raid.id.parse::<u32>().unwrap(),
        };

        for party in self.data.raid.parties.iter() {

            let party_id = self.id_generator.resolve_u32(&party.id);

            let mut simulator_party = SimulatorParty {
                id: party_id,  
                buffs: HashMap::new(),
                members: vec![]
            };

            let mut members = vec![];

            for EncounterTemplatePartyMember {
                attack_power,
                crit_rate,
                crit_damage,
                cooldown_reduction,
                class_id,
                gear_score,
                hp,
                name
            } in party.members.clone() {

                let name = self.id_generator.resolve_nickname(&name);
                
                let class_id = match class_id.as_str() {
                    "<dps>" => {
                        let class = Class::DPS().iter().choose(&mut self.rng).unwrap();
                        *class
                    },
                    "<support>" => {
                        let class = Class::SUPPORT().iter().choose(&mut self.rng).unwrap();
                        *class
                    },
                    value => {
                        let class = Class::from_str(value)?;
                        class
                    }
                };

               let gear_level = match parse_range(&gear_score) {
                    Some((from, to)) => {
                        self.rng.gen_range(from..=to) as f32
                    },
                    None => gear_score.parse::<f32>()?,
                };

                let mut character_id: u64 = self.id_generator.new_u64();

                let member = PKTPartyInfoInner {
                    name: name.clone(),
                    class_id: class_id as u32,
                    character_id,
                    gear_level
                };

                members.push(member);

                let mut player_id = self.rng.random();

                while self.ids.contains(&player_id) {
                    player_id = self.rng.random();
                }

                let args = SimulatorPlayerCreateArgs {
                    id: player_id,                    
                    character_id,
                    class_id,
                    attack_power,
                    crit_rate,
                    crit_damage,
                    cooldown_reduction
                };
                let simulator_player = SimulatorPlayerBase::new(args);
                simulator_party.members.push(simulator_player);

                if name == self.data.local_player {

                    let packet = encode_init_pc(
                        player_id,
                        name,
                        class_id as u32,
                        gear_level,
                        character_id,
                        hp as i64);
                    self.packets.push(packet);

                    continue;
                }

                let packet = encode_new_pc(
                    player_id,
                    name,
                    class_id as u32,
                    gear_level,
                    character_id,
                    hp as i64,
                );

                self.packets.push(packet);
            }

            let packet = encode_party_info(party_id, raid_id, members);

            self.packets.push(packet);

            self.parties.push(simulator_party);
        }

        Ok(())
    }

    pub fn to_skill_damage_packet(result: SimulatorPlayerSkillConsumeResult) -> Vec<Packet> {

        let mut packets = vec![];

        let SimulatorPlayerSkillConsumeResult {
            deals_damage,
            target_id,
            tripod_index,
            tripod_level,
            skill_id,
            source_id,
            damage,
            current_boss_hp,
            max_boss_hp,
            hit_flag,
            hit_option,
            buffs,
        } = result;

        let packet = encode_skill_start_notify(source_id, skill_id, tripod_index, tripod_level);
        packets.push(packet);

        let packet = encode_skill_cast_notify(source_id, skill_id);
        packets.push(packet);

        if deals_damage {
            let packet = encode_skill_damage_packet(
                source_id,
                skill_id,
                target_id,
                hit_flag,
                hit_option,
                current_boss_hp,
                max_boss_hp,
                damage
            );
  
            packets.push(packet);
        }

        packets
    }

    pub fn remove_expired_buffs(&mut self, now: DateTime<Utc>, packets: &mut Vec<Packet>) {

        for party in &mut self.parties {
            for player in &mut party.members {
                let expired_buffs: Vec<(u32, u32)> = player.buffs().iter()
                    .filter(|(_, buff)| buff.expires_on <= now)
                    .map(|(&buff_id, buff)| (buff_id, buff.id))
                    .collect();

                for (buff_id, buff_instance_id) in expired_buffs {
                    let packet = encode_status_effect_remove_notify(
                        player.id(),
                        0,
                        buff_instance_id,
                    );
                    packets.push(packet);

                    player.buffs_mut().remove(&buff_id);
                }
            }

            let expired_party_buffs: Vec<(u32, u64, u32)> = party.buffs.iter()
                .filter(|(_, buff)| buff.expires_on <= now)
                .map(|(&buff_id, buff)| (buff_id, buff.target_id, buff.id))
                .collect();

            for (buff_id, target_id, buff_instance_id) in expired_party_buffs {

                let packet = encode_party_status_effect_remove_notify(
                    target_id,
                    0,
                    buff_instance_id,
                );
                packets.push(packet);

                party.buffs.remove(&buff_id);
            }

            let expired_boss_debuffs: Vec<(u32, u32)> = self.context.boss_debuffs.iter()
                .filter(|(_, buff)| buff.expires_on <= now)
                .map(|(&buff_id, buff)| (buff_id, buff.id))
                .collect();

            for (buff_id, buff_instance_id) in expired_boss_debuffs {
                let packet = encode_status_effect_remove_notify(
                    self.context.boss_id,
                    0,
                    buff_instance_id,
                );
                packets.push(packet);

                self.context.boss_debuffs.remove(&buff_id);
            }
        }
    }

    pub fn tick(&mut self, now: DateTime<Utc>) -> Option<Vec<Packet>> {

        let mut packets = vec![];

        self.remove_expired_buffs(now, &mut packets);

        self.packets.extend(packets);

        if !self.packets.is_empty() {
            return Some(self.packets.drain(..).collect())
        }

        if self.context.current_boss_hp <= 0 {
            return None;
        }

        let mut packets = vec![];

        for party in &mut self.parties {
            for player in &mut party.members {

                let SimulatorPlayerStats {
                    attack_power,
                    crit_rate,
                    crit_damage,
                    ..
                } = player.get_combat_stats();

                let skill = player.get_available_skill(now);

                let skill = match skill {
                    Some(skill) => skill,
                    None => continue,
                };

                let result = skill.consume(
                    attack_power,
                    crit_rate,
                    crit_damage,
                    &mut self.context,
                    now);
                sleep(Duration::from_millis(250));

                let skill_packets = Self::to_skill_damage_packet(result.clone());

                packets.extend(skill_packets);

                Self::apply_buffs(
                    player.id(),
                    player.character_id(),
                    &mut self.id_generator,
                    &mut self.context,
                    &mut packets,
                    &mut player.buffs_mut(),
                    &mut party.buffs,
                    result.buffs.clone(),
                    now);
            }
        }

        Some(packets)
    }

    pub fn apply_buffs(
        player_id: u64,
        character_id: u64,
        id_generator: &mut IdGenerator,
        context: &mut SimulatorContext,
        packets: &mut Vec<Packet>,
        player_buffs: &mut HashMap<u32, SimulatorSkillBuff>,
        party_buffs: &mut HashMap<u32, SimulatorSkillBuff>,
        buffs: Vec<SimulatorPlayerSkillBuff>,
        now: DateTime<Utc>) {
        for buff in buffs.clone() {
            match buff.category {
                SimulatorPlayerSkillBuffCategory::Buff => {
                    match buff.target {
                        SimulatorPlayerSkillBuffTarget::SelfTarget => {
                            let id = id_generator.new_u32();
                            
                            let packet = encode_status_effect_add_notify(
                                player_id,
                                player_id,
                                buff.id,
                                id,
                                buff.duration.as_seconds_f32());
                            packets.push(packet);

                            match player_buffs.entry(buff.id) {
                                Entry::Occupied(mut entry) => {
                                    let entry = entry.get_mut();
                                    entry.expires_on = now + buff.duration;
                                },
                                Entry::Vacant(entry) => {
                                    let expires_on = now + buff.duration;
                                    let value = SimulatorSkillBuff::new(
                                        id,
                                        player_id,
                                        buff,
                                        expires_on);
                                    entry.insert(value);
                                },
                            }
                        }
                        SimulatorPlayerSkillBuffTarget::SelfParty => {
                              let id = id_generator.new_u32();
                            let packet = encode_party_status_effect_add_notify(
                                character_id,
                                vec![(player_id, buff.id, id, buff.duration.as_seconds_f32())]);
                            packets.push(packet);

                            match party_buffs.entry(buff.id) {
                                Entry::Occupied(mut entry) => {
                                    let entry = entry.get_mut();
                                    entry.expires_on = now + buff.duration;
                                },
                                Entry::Vacant(entry) => {
                                    let expires_on = now + buff.duration;
                                    let value = SimulatorSkillBuff::new(
                                        id,
                                        character_id,
                                        buff,
                                        expires_on);
                                    entry.insert(value);
                                },
                            }
                        },
                    }
                },
                SimulatorPlayerSkillBuffCategory::Debuff => {
                      let id = id_generator.new_u32();

                    let packet = encode_status_effect_add_notify(
                        context.boss_id,
                        player_id,
                        buff.id,
                        id,
                        buff.duration.as_seconds_f32());

                    packets.push(packet);

                    match context.boss_debuffs.entry(buff.id) {
                        Entry::Occupied(mut entry) => {
                            let entry = entry.get_mut();
                            entry.expires_on = now + buff.duration;
                        },
                        Entry::Vacant(entry) => {
                                let expires_on = now + buff.duration;
                                let value = SimulatorSkillBuff::new(
                                    id,
                                    context.boss_id,
                                    buff,
                                    expires_on);
                                entry.insert(value);
                        },
                    }
                },
            }
        }
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