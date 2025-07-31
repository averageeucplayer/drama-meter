use anyhow::*;
use log::info;
use std::sync::mpsc::{self, Receiver};
use meter_core::packets::opcodes::Pkt;
use crate::sniffer::PacketSniffer;


pub struct WindivertSniffer {

}

impl PacketSniffer for WindivertSniffer {
    fn start(&mut self, port: u16, region_file_path: String) -> Result<Receiver<(Pkt, Vec<u8>)>> {
        let (tx, rx) = mpsc::channel::<(Pkt, Vec<u8>)>();
        info!("started windivert sniffer");
        Ok((rx))
    }
}

impl WindivertSniffer {
    pub fn new() -> Self {
        Self {}
    }
}