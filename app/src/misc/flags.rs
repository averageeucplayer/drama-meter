use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use log::*;
use tauri::{AppHandle, Emitter, EventTarget, Listener, Runtime};


pub struct FlagsManager {
    reset: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    save: Arc<AtomicBool>,
    boss_only_damage: Arc<AtomicBool>,
    emit_details: Arc<AtomicBool>,
}

impl FlagsManager {
    pub fn new() -> Self {

        let reset = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let save = Arc::new(AtomicBool::new(false));
        let boss_only_damage = Arc::new(AtomicBool::new(false));
        let emit_details = Arc::new(AtomicBool::new(false));

        Self {
            reset,
            pause,
            save,
            boss_only_damage,
            emit_details
        }
    }

    pub fn setup_listeners(&self, app_handle: &AppHandle) {
        let reset = self.reset.clone();
        let save = self.save.clone();
        let pause = self.pause.clone();
        let boss_only_damage = self.boss_only_damage.clone();
        let emit_details = self.emit_details.clone();

        app_handle.listen_any("reset-request", {
            let app_handle = app_handle.clone();
            move |_event| {
                reset.store(true, Ordering::Relaxed);
                info!("resetting meter");
                app_handle.emit_to(EventTarget::Any, "reset-encounter", "").unwrap();
            }
        });

        app_handle.listen_any("save-request", {
            let app_handle = app_handle.clone();
            move |_event| {
                save.store(true, Ordering::Relaxed);
                info!("manual saving encounter");
                app_handle.emit_to(EventTarget::Any, "save-encounter", "").unwrap();
            }
        });

        app_handle.listen_any("pause-request", {
            let app_handle = app_handle.clone();
            move |_event| {
                let prev = pause.fetch_xor(true, Ordering::Relaxed);
                if prev {
                    info!("unpausing meter");
                } else {
                    info!("pausing meter");
                }
                
                app_handle.emit_to(EventTarget::Any, "pause-encounter", "").unwrap();
            }
        });

        app_handle.listen_any("boss-only-damage-request", {
            
            move |event| {
                let payload = event.payload();

                if let Some(bod) = (payload != "").then(|| payload) {
                    if bod == "true" {
                        boss_only_damage.store(true, Ordering::Relaxed);
                        info!("boss only damage enabled")
                    } else {
                        boss_only_damage.store(false, Ordering::Relaxed);
                        info!("boss only damage disabled")
                    }
                }
            }
        });

        app_handle.listen_any("emit-details-request", {
            let emit_clone = emit_details.clone();
            move |_event| {
                let prev = emit_clone.fetch_xor(true, Ordering::Relaxed);
                if prev {
                    info!("stopped sending details");
                } else {
                    info!("sending details");
                }
            }
        });
    }

    pub fn set_boss_only_damage(&self) {
        self.boss_only_damage.store(true, Ordering::Relaxed);
    }

    pub fn invoked_reset(&self) -> bool {
        let has_reset = self.reset.load(Ordering::Relaxed);
        
        if has_reset {
            self.reset.store(false, Ordering::Relaxed);
        }

        has_reset
    }

    pub fn invoked_save(&self) -> bool {
        let has_saved = self.save.load(Ordering::Relaxed);
        
        if has_saved {
            self.save.store(false, Ordering::Relaxed);
        }

        has_saved
    }

    pub fn toggled_boss_only_damage(&self) -> bool {
        self.boss_only_damage.load(Ordering::Relaxed)
    }

    pub fn is_paused(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
    }
}