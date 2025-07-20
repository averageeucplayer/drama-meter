use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use log::info;
use meter_core::packets::common::SkillMoveOptionData;

use crate::{models::StatusEffectDetails, models::{IncapacitatedEvent, IncapacitationEventType, Skill, SkillStats}};

#[derive(Debug, Default)]
pub struct Player {
    pub id: u64,
    pub is_local: bool,
    pub name: String,
    pub class_id: u32,
    pub gear_level: f32,
    pub character_id: u64,
    pub game_stats: HashMap<u8, i64>,
    pub encounter_stats: PlayerStats,
    pub incapacitations: Vec<IncapacitatedEvent>,
}

impl Player {

    pub fn on_shield_received(&mut self, buff_id: u32, value: u64) {
    
    }

    pub fn on_shield_given(&mut self, buff_id: u32, value: u64) {
      
    }

    pub fn shorten_incapacitation(&mut self, recorded_on: DateTime<Utc>) {
         for ongoing_event in self.incapacitations
                .iter_mut()
                .rev()
                .take_while(|x| x.recorded_on + x.duration > recorded_on)
                .filter(|x| x.event_type == IncapacitationEventType::FallDown)
            {
                info!(
                    "Shortening down duration from {} to {} because of getup skill",
                    ongoing_event.duration,
                    recorded_on - ongoing_event.recorded_on
                );
                ongoing_event.duration = recorded_on - ongoing_event.recorded_on;
            }
    }

    pub fn on_cc_applied(&mut self, status_effect: &StatusEffectDetails) {

        // expiration delay is zero or negative for infinite effects. Instead of applying them now,
        // only apply them after they've been removed (this avoids an issue where if we miss the removal
        // we end up applying a very long incapacitation)
        
        
        if status_effect.is_infinite() {
            return;
        }

        let duration_ms = status_effect.expiration_delay * 1000.0;
        let new_event = IncapacitatedEvent {
            recorded_on: status_effect.timestamp,
            duration: Duration::milliseconds(duration_ms as i64),
            event_type: IncapacitationEventType::CrowdControl,
        };
        
        info!(
            "Player {} will be status-effect incapacitated for {}ms by buff {}",
            self.name, duration_ms, status_effect.status_effect_id
        );

        self.incapacitations.push(new_event);
    }

      pub fn on_abnormal_move( &mut self, recorded_on: DateTime<Utc>, movement: &SkillMoveOptionData) {
       
        // only count movement events that would result in a knockup
        let Some(down_time) = movement.down_time else {
            return;
        };

        // todo: unclear if this is fully correct. It's hard to debug this, but it seems roughly accurate
        // if this is not accurate, we should probably factor out the stand_up_time and instead add in the
        // animation duration of the standup action for each class (seems to be 0.9s)
        let total_incapacitated_time = down_time
            + movement.move_time.unwrap_or_default()
            + movement.stand_up_time.unwrap_or_default();
        let incapacitated_time_ms = (total_incapacitated_time * 1000.0) as i64;

        // see if we have a previous incapacitation event that is still in effect (i.e. the player was knocked up again before
        // they could stand up), in which case we should shorten the previous event duration to the current timestamp
        let prev_incapacitation = self.incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.recorded_on + e.duration > recorded_on) // stop as soon as we only hit expired events
            .find(|x| x.event_type == IncapacitationEventType::FallDown); // find an unexpired one that was caused by an abnormal move
        if let Some(prev_incapacitation) = prev_incapacitation {
            info!(
                "Shortening down duration from {} to {} because of new abnormal move",
                prev_incapacitation.duration,
                recorded_on - prev_incapacitation.recorded_on
            );
            prev_incapacitation.duration = recorded_on - prev_incapacitation.recorded_on;
        }

        let new_event = IncapacitatedEvent {
            recorded_on,
            duration: Duration::milliseconds(incapacitated_time_ms),
            event_type: IncapacitationEventType::FallDown,
        };
        
        self.incapacitations.push(new_event);

        info!(
            "Player {} will be incapacitated for {}ms",
            self.name, incapacitated_time_ms
        );
    }

    pub fn on_cc_removed(&mut self, recorded_on: DateTime<Utc>, status_effect: &StatusEffectDetails) {
        if status_effect.is_infinite() {
            // this status effect was infinite, meaning we didn't apply it on_cc_applied
            // apply it now retroactively, then sort the events to ensure that our sorted
            // invariant does not get violated
            let duration = recorded_on - status_effect.timestamp;
            let new_event = IncapacitatedEvent {
                recorded_on: status_effect.timestamp,
                duration,
                event_type: IncapacitationEventType::CrowdControl,
            };
            info!(
                "Player {} was incapacitated by an infinite status effect buff for {}ms",
                self.name, duration.num_milliseconds()
            );

            self.incapacitations.push(new_event);
            self.incapacitations.sort_by_key(|x| x.recorded_on);
            return;
        }

        // we use the application timestamp as the key. Attempt to find all buff instances that started
        // at this time and cap their duration to the current timestamp
        for event in self.incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.recorded_on + e.duration > recorded_on)
        {
            if event.event_type == IncapacitationEventType::CrowdControl
                && event.recorded_on == status_effect.timestamp
            {
                info!(
                    "Removing status-effect {} incapacitation for player {} (shortened {}ms to {}ms)",
                    status_effect.status_effect_id,
                    self.name,
                    event.duration,
                    recorded_on - event.recorded_on
                );
                event.duration = recorded_on - event.recorded_on;
            }
        }
    }

    pub fn trim_incapacitations_to_death_time(&mut self) {
        let death_time = self.encounter_stats.death_log.recorded_on;

        self.incapacitations
            .iter_mut()
            .rev()
            .take_while(|x| x.recorded_on + x.duration > death_time)
            .for_each(|x| {
                // Cap duration so incapacitation does not exceed death time.
                x.duration = x.recorded_on - death_time;
            });
    }
}

#[derive(Debug, Default)]
pub struct PlayerStats {
    pub is_dead: bool,
    pub skills: HashMap<u32, Skill>,
    pub damage_stats: PlayerDamageStats,
    pub skill_stats: SkillStats,
    pub death_log: DeathLog,
    pub current_hp: i64,
    pub max_hp: i64,
    pub hyper_awakening_damage: i64,
    pub damage_taken: i64,
    pub buffed_by: HashMap<u32, i64>,
    pub debuffed_by: HashMap<u32, i64>,
    pub buffed_by_support: i64,
    pub buffed_by_identity: i64,
    pub debuffed_by_support: i64,
    pub buffed_by_hat: i64,
    pub crit_damage: i64,
    pub back_attack_damage: i64,
    pub front_attack_damage: i64,
    pub shields_given: u64,
    pub shields_received: u64,
    pub shields_given_by: HashMap<u32, u64>,
    pub shields_received_by: HashMap<u32, u64>,
}

#[derive(Debug, Default)]
pub struct DeathLog {
    pub count: i64,
    pub recorded_on: DateTime<Utc>
}

#[derive(Debug, Default)]
pub struct PlayerDamageStats {
    pub dealt: i64,
    pub crit: i64,
    pub back_attack: i64,
    pub front_attack: i64,
    pub absorbed: u64,
    pub absorbed_on_others: u64,
    pub absorbed_by: HashMap<u32, u64>,
    pub absorbed_on_others_by: HashMap<u32, u64>,
}