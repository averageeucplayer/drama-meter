use anyhow::*;
use chrono::Utc;
use log::info;
use rand::seq::IteratorRandom;
use std::{fs, path::{Path, PathBuf}, sync::mpsc::{self, Receiver, Sender}, thread::{sleep, spawn, JoinHandle}, time::Duration};
use meter_core::packets::opcodes::Pkt;
use crate::{simulator::{Packet, Simulator}, sniffer::PacketSniffer};

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

pub fn process(tx: Sender<Packet>, template_path: PathBuf, destination_dir: PathBuf) {
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
                sleep(Duration::from_millis(500));
            }
            sleep(Duration::from_millis(500));
            now = Utc::now();
        }

        let file_name = path.file_name().unwrap();
        let destination_path = destination_dir.join(file_name);
        move_to_processed(&path, &destination_path);

        sleep(next_encounter_delay)
    }
}


pub struct FakeSniffer {
    handle: Option<JoinHandle<()>>,
}

impl PacketSniffer for FakeSniffer {
    fn start(&mut self, port: u16, region_file_path: String) -> Result<Receiver<(Pkt, Vec<u8>)>> {
        let (tx, rx) = mpsc::channel::<(Pkt, Vec<u8>)>();
        info!("started fake sniffer");

        let handle = spawn(|| {
            let current_exec = std::env::current_exe().unwrap();
            let parent_path = current_exec.parent().unwrap();
            let template_path = parent_path.join("assets").join("templates");
            let destination_dir = parent_path.join("processed");
            info!("running fake sniffer template_path: {} processed_dir: {}",
                template_path.to_string_lossy(),
                destination_dir.to_string_lossy());

            fs::create_dir_all(&destination_dir).unwrap();

            process(tx, template_path, destination_dir);
        });

        self.handle = Some(handle);

        Ok((rx))
    }
}

impl FakeSniffer {
    pub fn new() -> Self {
        Self { handle: None }
    }
}