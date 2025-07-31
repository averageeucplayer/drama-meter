use chrono::{Duration, Timelike};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplate {
    pub boss: EncounterTemplateBoss,
    pub sidereals: Vec<EncounterTemplateSidereal>,
    pub raid: EncounterTemplateRaid,
    pub local_player: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplateBoss {
    pub id: u32,
    pub level: u16,
    pub hp: f64,
    pub summons: Vec<EncounterTemplateBossSummons>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplateSidereal {
    pub id: u32,
    pub damage: i64,
    #[serde(deserialize_with = "parse_duration_hms")]
    pub appears_after: Duration,
    #[serde(deserialize_with = "parse_duration_hms")]
    pub expires_after: Duration,
    pub skill_id: u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterTemplateBossSummons {
    pub id: u32,
    pub hp: f64,
    pub appears_after_death: Option<bool>,
    pub appears_after_hp_bar: Option<u32>,
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

fn parse_duration_hms<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let parsed = chrono::NaiveTime::parse_from_str(s, "%H:%M:%S")
        .map_err(serde::de::Error::custom)?;
    Ok(Duration::seconds(parsed.num_seconds_from_midnight() as i64))
}