use std::fs::File;

use hashbrown::{HashMap, HashSet};
use once_cell::sync::Lazy;
use serde::Deserialize;
use strum::VariantArray;
use tokio::task::{self, JoinHandle};

use crate::models::*;


pub static NPC_DATA: Lazy<HashMap<u32, Npc>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/Npc.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static SKILL_DATA: Lazy<HashMap<u32, SkillData>> = Lazy::new(|| {
     unsafe {
        let reader = File::open("assets/data/Skill.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static SKILL_EFFECT_DATA: Lazy<HashMap<u32, SkillEffectData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/SkillEffect.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static SKILL_BUFF_DATA: Lazy<HashMap<u32, SkillBuffData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/SkillBuff.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static COMBAT_EFFECT_DATA: Lazy<HashMap<i32, CombatEffectData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/CombatEffect.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static ENGRAVING_DATA: Lazy<HashMap<u32, EngravingData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/Ability.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static ARK_PASSIVE_DATA: Lazy<HashMap<u32, ArkPassiveInfo>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/ArkPassive.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static ARK_PASSIVE_ID_TO_SPEC: Lazy<HashMap<u32, String>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/ArkPassiveIdSpec.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static CARD_MAP: Lazy<HashMap<u32, Card>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/CardMap.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static BOSS_HP_MAP: Lazy<HashMap<String, u32>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/BossHpMap.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static CLASS_NAMES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    Class::VARIANTS.into_iter().map(|pr| pr.as_ref()).collect()
});

pub static CLASS_MAP: Lazy<HashMap<u32, &'static str>> = Lazy::new(|| {
    Class::VARIANTS.into_iter().map(|pr| (*pr as u32, pr.as_ref())).collect()
});

pub static REVERSE_CLASS_MAP: Lazy<HashMap<&'static str, u32>> = Lazy::new(|| {
    Class::VARIANTS.into_iter().map(|pr| (pr.as_ref(), *pr as u32)).collect()
});

pub static ESTHER_DATA: Lazy<Vec<Esther>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/Esther.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static ESTHER_BY_NPC_ID: Lazy<HashMap<u32, Esther>> = Lazy::new(|| {
    let data: Vec<Esther> = unsafe {
        let reader = File::open("assets/data/Esther.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    };

    data
        .into_iter()
        .flat_map(|esther| {
            esther.npc_ids.clone().into_iter().map(move |npc_id| (npc_id, esther.clone()))
        })
        .collect()
});

pub static ENCOUNTER_MAP: Lazy<HashMap<String, HashMap<String, Vec<String>>>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/Encounters.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static RAID_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let encounters: HashMap<String, HashMap<String, Vec<String>>> = unsafe {
        let reader = File::open("assets/data/Encounters.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    };
    encounters
        .values()
        .flat_map(|raid| raid.iter())
        .flat_map(|(gate, bosses)| bosses.iter().map(move |boss| (boss.clone(), gate.clone())))
        .collect()
});

pub static VALID_ZONES: Lazy<HashSet<u32>> = Lazy::new(|| {
    let valid_zones = [
        30801, 30802, 30803, 30804, 30805, 30806, 30807, 30835, 37001, 37002, 37003, 37011,
        37012, 37021, 37022, 37031, 37032, 37041, 37042, 37051, 37061, 37071, 37072, 37081,
        37091, 37092, 37093, 37094, 37101, 37102, 37111, 37112, 37121, 37122, 37123, 37124,
        308010, 308011, 308012, 308014, 308015, 308016, 308017, 308018, 308019, 308020, 308021,
        308022, 308023, 308024, 308025, 308026, 308027, 308028, 308029, 308030, 308037, 308039,
        308040, 308041, 308042, 308043, 308044, 308239, 308339, 308410, 308411, 308412, 308414,
        308415, 308416, 308417, 308418, 308419, 308420, 308421, 308422, 308423, 308424, 308425,
        308426, 308428, 308429, 308430, 308437, 309020, 30865, 30866,
    ];
    valid_zones.iter().cloned().collect()
});

pub static STAT_TYPE_MAP: Lazy<HashMap<String, u32>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/StatTypes.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static IDENTITY_CATEGORY: Lazy<HashMap<String, i32>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("assets/data/IdentityCategory.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static GEM_SKILL_MAP: Lazy<HashMap<u32, Vec<u32>>> = Lazy::new(|| {
    unsafe {
        use serde::Deserialize;

        let reader = File::open("assets/data/GemSkillGroup.json").unwrap_unchecked();
        let data: HashMap<String, (String, String, Vec<u32>)> = serde_json::from_reader(reader).unwrap_unchecked();

        data
            .into_iter()
            .filter_map(|(key, entry)| key.parse::<u32>().ok().map(|id| (id, entry.2)))
            .collect()
    }
});

pub static GUARDIAN_RAID_BOSSES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "Drextalas",
        "Skolakia",
        "Argeos",
        "Veskal",
        "Gargadeth",
        "Sonavel",
        "Hanumatan",
        "Kungelanium",
        "Deskaluda"
    ]
});

pub struct AssetsPreloader(Option<JoinHandle<()>>);

impl AssetsPreloader {
    pub fn new() -> Self {
        let mut data = Self(None);
        data.load();

        data
    }

    pub async fn wait_for_load(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.await;
        }
    }

    fn load(&mut self) {
        let handle = task::spawn_blocking(|| {
            let _ = COMBAT_EFFECT_DATA.iter().next();
            let _ = ENGRAVING_DATA.iter().next();
            let _ = ARK_PASSIVE_DATA.iter().next();
            let _ = ARK_PASSIVE_ID_TO_SPEC.iter().next();
            let _ = CARD_MAP.iter().next();
            let _ = BOSS_HP_MAP.iter().next();
            let _ = CLASS_NAMES.len();
            let _ = CLASS_MAP.iter().next();
            let _ = REVERSE_CLASS_MAP.iter().next();
            let _ = ESTHER_DATA.len();
            let _ = ESTHER_BY_NPC_ID.iter().next();
            let _ = ENCOUNTER_MAP.iter().next();
            let _ = RAID_MAP.iter().next();
            let _ = VALID_ZONES.len();
            let _ = STAT_TYPE_MAP.iter().next();
            let _ = IDENTITY_CATEGORY.iter().next();
            let _ = GEM_SKILL_MAP.iter().next();
            let _ = GUARDIAN_RAID_BOSSES.len();
            let _ = GEM_SKILL_MAP.len();
        });

        self.0 = Some(handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_load_assets() {
        // assert!(NPC_DATA.iter().next().is_some());
        // assert!(SKILL_DATA.iter().next().is_some());
        // assert!(SKILL_EFFECT_DATA.iter().next().is_some());
        //assert!(SKILL_BUFF_DATA.iter().next().is_some());
        //assert!(COMBAT_EFFECT_DATA.iter().next().is_some());
        assert!(ENGRAVING_DATA.iter().next().is_some());
        assert!(ARK_PASSIVE_DATA.iter().next().is_some());
        assert!(ESTHER_BY_NPC_ID.iter().next().is_some());
        assert!(ESTHER_DATA.iter().next().is_some());
        assert!(RAID_MAP.iter().next().is_some());
        assert!(VALID_ZONES.iter().next().is_some());
        assert!(STAT_TYPE_MAP.iter().next().is_some());
        assert!(IDENTITY_CATEGORY.iter().next().is_some());

        // assert!(CARD_MAP.iter().next().is_some());

        // let entry = CARD_MAP.iter().next().unwrap();

        // println!("{entry:?}");
    }

}