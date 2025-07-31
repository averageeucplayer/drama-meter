mod fake;
mod windivert;

use anyhow::*;
use meter_core::packets::opcodes::Pkt;
use std::{error::Error, sync::mpsc::{self, Receiver, Sender}};

pub use fake::FakeSniffer;
pub use windivert::WindivertSniffer;

pub trait PacketSniffer : Send + Sync {
    fn start(&mut self, port: u16, region_file_path: String) -> Result<Receiver<(Pkt, Vec<u8>)>>;
}