use crate::core::utils::*;
use crate::misc::app_context::AppContext;
use crate::database::Database;
use crate::misc::flags::FlagsManager;
use crate::core::encounter_state::EncounterState;
use crate::core::handler::handle;
use crate::core::stats_api::{StatsApi, API_URL};
use crate::misc::local::LocalManager;
use crate::misc::region::RegionManager;
use crate::misc::settings::Settings;
use crate::sniffer::PacketSniffer;
use anyhow::Result;
use chrono::{Duration, Utc};
use log::{info, warn};
use meter_core::packets::opcodes::Pkt;
use tokio::runtime::Runtime;

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Emitter, EventTarget};

pub struct BackgroundWorkerArgs {
    pub packet_sniffer: Box<dyn PacketSniffer>,
    pub version: String,
    pub app: AppHandle,
    pub database: Arc<Database>,
    pub context: Arc<AppContext>,
    pub port: u16,
    pub settings: Settings
}

pub struct BackgroundWorker(Option<JoinHandle<()>>);

impl BackgroundWorker {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn run(&mut self) {
        let handle = thread::spawn(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {

            });
        });

        self.0 = Some(handle);
    }

    async fn run_inner(args: BackgroundWorkerArgs) -> Result<()> {

        let BackgroundWorkerArgs {
            app,
            context,
            database,
            packet_sniffer,
            port,
            settings,
            version,
        } = args;     

        let mut state: EncounterState = EncounterState::new(version);
        let mut region_manager = RegionManager::new(context.region_path.clone());
        let flags_manager = FlagsManager::new();
        let mut local_manager = LocalManager::new(context.database_path.clone())?;
        let mut stats_api = Arc::new(StatsApi::new());

        let rx = packet_sniffer.start(port, context.region_path.to_string_lossy().to_string())?;

        let damage_handler = meter_core::decryption::DamageEncryptionHandler::new();
        damage_handler.start()?;
        
        if settings.general.boss_only_damage {
            flags_manager.set_boss_only_damage();
            info!("boss only damage enabled")
        }

        if settings.general.low_performance_mode {
            state.update_interval = Duration::milliseconds(1500);
            info!("low performance mode enabled")
        }

        state.region = region_manager.get();

        flags_manager.setup_listeners(&app);
        let update_interval = state.update_interval.to_std().unwrap();

        loop {
            let (op, data) = match rx.recv_timeout(update_interval) {
                Ok(result) => result,
                Err(_) => (Pkt::Void, vec![]),
            };

            if flags_manager.invoked_reset() {
                state.soft_reset(true);
            }

            if flags_manager.is_paused() {
                continue;
            }

            if flags_manager.invoked_save() {
                state.party_info = state.get_party();

                if let Some(model) = state.get_encounter(true) {  
                    save_to_db(app.clone(), stats_api.clone(), database.clone(), model);
                    state.saved = true;
                    state.is_resetting = true;
                }
            }

            if flags_manager.toggled_boss_only_damage() {
                state.boss_only_damage = true;
            } else {
                state.boss_only_damage = false;
            }

            let now = Utc::now();
            if let Err(err) = handle(
                now,
                op,
                app.clone(),
                &mut state,
                &data,
                &damage_handler,
                &mut local_manager,
                &mut region_manager,
                &stats_api,
                database.clone()) {
                warn!("An error occurred whilst parsing {}", err);
            }

            if let Some(data) = state.get_ongoing_encounter(now) {
                app.emit_to(EventTarget::Any, "encounter-update", Some(&data))?;
            }

            if state.is_resetting {
                state.soft_reset(true);
            }
        }

        Ok(())
    }
}