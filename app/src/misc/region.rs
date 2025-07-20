use std::path::PathBuf;


pub struct RegionManager(PathBuf);

impl RegionManager {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn get(&self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }
}

// fn get_and_set_region(path: &str, state: &mut EncounterState) {
//     match std::fs::read_to_string(path) {
//         Ok(region) => {
//             state.region = Some(region.clone());
//             state.encounter.region = Some(region);
//         }
//         Err(_) => {
//             // warn!("failed to read region file. {}", e);
//         }
//     }
// }