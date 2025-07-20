use crate::packets::structures::SkillDamageEvent;

pub struct DamageEncryptionHandler{}

impl DamageEncryptionHandler{
    pub fn new() -> Self {
        Self {}
    }

    pub fn decrypt_damage_event(&self, event: &mut SkillDamageEvent) -> bool {
        true
    }

    pub fn update_zone_instance_id(&self, channel_id: u32) {

    }

    pub fn start(&self) -> anyhow::Result<()> {
        Ok(())
    }
}