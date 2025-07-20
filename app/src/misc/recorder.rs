use std::{fs::File, io::{self, Read, Write}, mem::transmute, path::Path};

use meter_core::packets::opcodes::Pkt;


pub struct Recorder(File);

impl Recorder {
     pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::options()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;
        Ok(Self(file))
    }

    pub fn write(&mut self, record_type: Pkt, data: &[u8]) -> io::Result<()> {
        
        self.0.write_all(&[record_type as u8])?;
        let len = (data.len() as u32).to_le_bytes();
        self.0.write_all(&len)?;
        self.0.write_all(data)?;

        Ok(())
    }

    pub fn read(&mut self) -> io::Result<Option<(Pkt, Vec<u8>)>> {

        let mut type_buf = [0u8; 1];
        let mut len_buf = [0u8; 4];

        if self.0.read_exact(&mut type_buf).is_err() {
            return Ok(None);
        }

        if self.0.read_exact(&mut len_buf).is_err() {
            return Ok(None);
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut data = vec![0u8; len];
        self.0.read_exact(&mut data)?;

        let raw: u8 = type_buf[0];
        let packet: Pkt = unsafe { transmute(raw) };

        Ok(Some((packet, data)))
    }
}