use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplate {
    pub boss: EncounterTemplateBoss,
    pub raid: EncounterTemplateRaid,
    pub local_player: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplateBoss {
    pub id: u32,
    pub level: u16,
    pub hp: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplateRaid {
    pub id: String,
    pub parties: Vec<EncounterTemplateParty>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplateParty {
    pub id: String,
    pub members: Vec<EncounterTemplatePartyMember>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncounterTemplatePartyMember {
    pub class_id: String,
    pub attack_power: u32,
    pub cooldown_reduction: f32,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub gear_score: String,
    pub hp: f32,
    pub name: String,
}