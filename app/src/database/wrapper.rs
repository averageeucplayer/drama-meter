use anyhow::*;
use hashbrown::HashMap;
use r2d2_sqlite::SqliteConnectionManager;
use serde_json::json;
use std::{cmp::Reverse, collections::BTreeMap, fs::{self, File}, hash::Hash, io::Read, path::PathBuf};
use log::*;
use rusqlite::{params, params_from_iter, Connection, Transaction};
use strfmt::strfmt;

use crate::{constants::*, database::{models::*, queries::*, utils::*}, core::{stats_api::PlayerStats, utils::*}, models::*, misc::utils::compress_json};

pub struct Database {
    path: PathBuf,
    pool: r2d2::Pool<SqliteConnectionManager>,
    is_new: bool
}

impl Database {
    pub fn new(path: PathBuf) -> Self {

        let is_new = !path.exists();

        let manager = SqliteConnectionManager::file(&path);
        let pool: r2d2::Pool<SqliteConnectionManager> = r2d2::Pool::new(manager).unwrap();

        Self {
            path,
            pool,
            is_new
        }
    }

    pub fn setup(&self, migration_path: PathBuf) -> Result<()> {
        
        if self.is_new {
            let mut connection = self.pool.get()?;
            
            let mut sql_files: Vec<_> = fs::read_dir(&migration_path)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.path().is_file() && entry.path().extension().map(|ext| ext == "sql").unwrap_or(false)
                })
                .collect();

            sql_files.sort_by_key(|entry| entry.path());

            for file in sql_files {
                let path = file.path();
                info!("Running migration: {:?}", path);

                let sql = fs::read_to_string(&path)?;
                connection.execute_batch(&sql)?;
            }
        }
        else {
            let mut connection = self.pool.get()?;
            let mut statement = connection.prepare("SELECT 1 FROM sqlite_master WHERE type=? AND name=?")?;
            let table_exists = statement.exists(["table", "encounter"])?;
        }

        Ok(())
    }

    pub async fn delete_encounters(&self, ids: Vec<i32>) -> Result<()> {
        let connection = self.pool.get()?;
        
        connection.execute("PRAGMA foreign_keys = ON;", params![])?;

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let placeholders_str = placeholders.join(",");

        let sql = format!("DELETE FROM encounter WHERE id IN ({})", placeholders_str);
        let mut statement = connection.prepare_cached(&sql)?;

        info!("deleting encounters: {:?}", ids);

        statement.execute(params_from_iter(ids))?;

        Ok(())
    }

    pub async fn delete_all_encounters(&self, keep_favorites: bool) -> Result<()> {
        let connection = self.pool.get()?;

        if keep_favorites {
            connection.execute(DELETE_FROM_ENCOUNTERS_IDS, [])?;
        } else {
            connection.execute(DELETE_FROM_ENCOUNTERS, [])?;
        }
        connection.execute("VACUUM", [])?;

        Ok(())
    }

    pub async fn get_sync_candidates(&self, force_resync: bool) -> Result<Vec<i32>> {
        let connection = self.pool.get()?;

        let query = if force_resync { "= '0'" } else { "IS NULL" };
        let mut args = std::collections::HashMap::new();
        args.insert("{}".to_string(), query.to_string());
        let query = strfmt(SELECT_ENCOUNTER_PREVIEW_JOIN_SYNC_FILTERED, &args)?;

        let mut statement = connection.prepare_cached(&query)?;
        let rows = statement.query_map([], |row| row.get(0))?;

        let mut ids = Vec::new();

        for id_result in rows {
            ids.push(id_result.unwrap_or(0));
        }

        Ok(ids)
    }

    pub async fn toggle_encounter_favorite(&self, id: i32) -> Result<()> {
        let connection = self.pool.get()?;

        let mut statement = connection.prepare_cached(UPDATE_ENCOUNTER_FAVOURITE)?;

        statement.execute(params![id])?;

        Ok(())
    }

    pub async fn insert_migration(&self, id: String) -> Result<()> {
        let connection = self.pool.get()?;

        connection.execute("PRAGMA foreign_keys = ON;", params![])?;
        
        let mut statement = connection.prepare_cached(DELETE_FROM_ENCOUNTERS_ID)?;

        info!("deleting encounter: {}", id);

        statement.execute(params![id])?;

        Ok(())
    }

    pub async fn delete_encounter(&self, id: String) -> Result<()> {
        let connection = self.pool.get()?;

        connection.execute("PRAGMA foreign_keys = ON;", params![])?;
        
        let mut statement = connection.prepare_cached(DELETE_FROM_ENCOUNTERS_ID)?;

        info!("deleting encounter: {}", id);

        statement.execute(params![id])?;

        Ok(())
    }

    pub async fn load_encounter(&self, id: String) -> Result<Encounter> {
        let connection = self.pool.get()?;

        let mut statement = connection.prepare_cached(SELECT_ENCOUNTER_JOIN_PREVIEW_BY_ID)?;

        let (mut encounter, compressed) = statement
            .query_row(params![id], |row| parse_encounter(row))
            .unwrap_or_else(|_| (Encounter::default(), false));

        let mut statement = connection.prepare_cached(SELECT_ENTITY_BY_ENCOUNTER_ID)?;

        let entity_iter = statement.query_map(params![id], |row| parse_entity(row, compressed))?;

        let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
        for entity in entity_iter.flatten() {
            entities.insert(entity.name.to_string(), entity);
        }

        let mut statement = connection.prepare_cached(SELECT_SYNC_LOG_BY_ENCOUNTER_ID)?;

        let sync: Result<String, rusqlite::Error> = statement.query_row(params![id], |row| row.get(0));
        encounter.sync = sync.ok();

        encounter.entities = entities;

        Ok(encounter)
    }

    pub async fn get_last_encounter(&self) -> Result<Option<i32>> {
        let connection = self.pool.get()?;

        let mut statement = connection
            .prepare_cached(SELECT_LATEST_ENCOUNTER_ID)?;

        let id = statement.query_row(params![], |row| row.get(0))?;

        Ok(id)
    }

    pub async fn load_encounters_preview(&self,
        page: i32,
        page_size: i32,
        search: String,
        filter: SearchFilter) -> Result<(Vec<EncounterPreview>, i32)> {
        let connection = self.pool.get()?;
        let mut sql_params = vec![];

        let join_clause = if search.len() > 2 {
            let escaped_search = search
                .split_whitespace()
                .map(|word| format!("\"{}\"", word.replace("\"", "")))
                .collect::<Vec<_>>()
                .join(" ");
            
            sql_params.push(escaped_search);
            "JOIN encounter_search(?) ON encounter_search.rowid = e.id"
        } else {
            ""
        };

        sql_params.push((filter.min_duration * 1000).to_string());

        let boss_filter = if !filter.bosses.is_empty() {
            let mut placeholders = "?,".repeat(filter.bosses.len());
            placeholders.pop(); // remove trailing comma
            sql_params.extend(filter.bosses);
            format!("AND e.current_boss IN ({})", placeholders)
        } else {
            "".to_string()
        };

        let raid_clear_filter = if filter.cleared {
            "AND cleared = 1"
        } else {
            ""
        };

        let favorite_filter = if filter.favorite {
            "AND favorite = 1"
        } else {
            ""
        };

        let boss_only_damage_filter = if filter.boss_only_damage {
            "AND boss_only_damage = 1"
        } else {
            ""
        };

        let difficulty_filter = if !filter.difficulty.is_empty() {
            sql_params.push(filter.difficulty);
            "AND difficulty = ?"
        } else {
            ""
        };

        let sort = format!("e.{}", filter.sort);

        let count_params = sql_params.clone();

        let mut args = std::collections::HashMap::new();
        args.insert("{join}".to_string(), join_clause.to_string());
        args.insert("{boss}".to_string(), boss_filter.to_string());
        args.insert("{clear}".to_string(), raid_clear_filter.to_string());
        args.insert("{favorite}".to_string(), favorite_filter.to_string());
        args.insert("{difficulty}".to_string(), difficulty_filter.to_string());
        args.insert("{boss_only_damage}".to_string(), boss_only_damage_filter.to_string());
        args.insert("{sort}".to_string(), sort);
        args.insert("{order}".to_string(), filter.order);

        let query = strfmt(SELECT_ENCOUNTER_PREVIEW, &args)?;

        let mut statement = connection.prepare_cached(&query)?;

        let offset = (page - 1) * page_size;

        sql_params.push(page_size.to_string());
        sql_params.push(offset.to_string());

        let sql_params = params_from_iter(sql_params);

        let encounter_iter = statement.query_map(sql_params, |row| parse_encounter_preview(row))?;

        let encounters: Vec<EncounterPreview> = encounter_iter.collect::<Result<_, _>>()?;

        let mut args = std::collections::HashMap::new();
        args.insert("{join}".to_string(), join_clause.to_string());
        args.insert("{boss}".to_string(), boss_filter.to_string());
        args.insert("{clear}".to_string(), raid_clear_filter.to_string());
        args.insert("{favorite}".to_string(), favorite_filter.to_string());
        args.insert("{boss_only_damage}".to_string(), boss_only_damage_filter.to_string());

        let query = strfmt(SELECT_ENCOUNTER_PREVIEW_FILTERED_COUNT, &args)?;

        let count: i32 = connection
            .query_row_and_then(&query, params_from_iter(count_params), |row| row.get(0))?;

        Ok((encounters, count))
    }

    pub async fn optimize(&self) -> Result<()> {
        let connection = self.pool.get()?;

        connection.execute_batch(INSERT_FTS5)?;
        info!("optimized database");

        Ok(())
    }

    pub async fn get_db_stats(&self, min_duration: i64) -> Result<(i32, i32)> {
        let connection = self.pool.get()?;

        let result: (i32, i32) = connection
            .query_row(SELECT_STATS, [], |row| {
                let result = (row.get::<_, i32>(0).unwrap(), row.get::<_, i32>(1).unwrap());
                rusqlite::Result::Ok(result)
            })?;

        Ok(result)
    }

    pub fn get_metadata(&self) -> Result<String> {

        let metadata = fs::metadata(&self.path)?;

        let size_in_bytes = metadata.len();
        let size_in_kb = size_in_bytes as f64 / 1024.0;
        let size_in_mb = size_in_kb / 1024.0;
        let size_in_gb = size_in_mb / 1024.0;

        let size_str = if size_in_gb >= 1.0 {
            format!("{:.2} GB", size_in_gb)
        } else if size_in_mb >= 1.0 {
            format!("{:.2} MB", size_in_mb)
        } else {
            format!("{:.2} KB", size_in_kb)
        };

        Ok(size_str)
    }

    pub async fn get_encounter_count(&self) -> Result<i32> {
        let connection = self.pool.get()?;

        let mut statement = connection.prepare_cached(SELECT_ENCOUNTER_PREVIEW_COUNT)?;

        let count: i32 = statement.query_row(params![], |row| row.get(0))?;

        Ok(count)
    }

    pub async fn delete_all_uncleared_encounters(&self, keep_favorites: bool) -> Result<()> {
        let connection = self.pool.get()?;

        if keep_favorites {
            connection.execute(DELETE_FROM_ENCOUNTERS_UNCLEARED_KEEP_FAVOURITE, [])?;
        } else {
            connection.execute(DELETE_FROM_ENCOUNTERS_UNCLEARED, [])?;
        }

        connection.execute("VACUUM", params![])?;

        Ok(())
    }

    pub async fn delete_encounters_below_min_duration(
        &self,
        min_duration: i64,
        keep_favorites: bool,
    ) -> Result<()> {
        let connection = self.pool.get()?;

        if keep_favorites {
            connection.execute(DELETE_FROM_ENCOUNTERS_BELOW_DURATION_KEEP_FAVOURITE, params![min_duration * 1000])?;
        } else {
            connection.execute(DELETE_FROM_ENCOUNTERS_BELOW_DURATION, params![min_duration * 1000])?;
        }

        connection.execute("VACUUM", params![])?;

        Ok(())
    }

    pub async fn insert_sync_log(&self, encounter: i32, upstream: String, failed: bool) -> Result<()> {
        let connection = self.pool.get()?;

        let sql_params = params![encounter, upstream, failed];
        connection.execute(INSERT_SYNC_LOG, sql_params)?;

        Ok(())
    }

    pub fn insert_data(&self, model: SaveToDb) -> Result<i64> {

        let SaveToDb {
            misc,
            duration,
            boss_only_damage,
            entities,
            started_on,
            updated_on,
            encounter_damage_stats,
            local_player,
            raid_difficulty,
            player_info,
            damage_log,
            raid_clear,
            cast_log,
            skill_cast_log,
            boss_hp_log,
            current_boss_name,
            ..
        } = model;

        let mut connection = self.pool.get()?;
        let tx = connection.transaction()?;
        
        let duration_seconds = (updated_on - started_on).num_seconds();
        let fight_start = started_on.timestamp_millis();
        let fight_end = updated_on.timestamp_millis();
        let compressed_boss_hp = compress_json(&boss_hp_log)?;
        let compressed_buffs = compress_json(&encounter_damage_stats.buffs)?;
        let compressed_debuffs = compress_json(&encounter_damage_stats.debuffs)?;
        let compressed_shields = compress_json(&encounter_damage_stats.applied_shield_buffs)?;

        let encounter_db = EncounterDb {
            last_combat_packet: updated_on.timestamp_millis(),
            total_damage_dealt: encounter_damage_stats.total_damage_dealt,
            top_damage_dealt: encounter_damage_stats.top_damage_dealt,
            total_damage_taken: encounter_damage_stats.total_damage_taken,
            top_damage_taken: encounter_damage_stats.top_damage_taken,
            dps: encounter_damage_stats.dps,
            compressed_buffs,
            compressed_debuffs,
            total_shielding: encounter_damage_stats.total_shielding,
            total_effective_shielding: encounter_damage_stats.total_effective_shielding,
            compressed_shields,
            misc: json!(misc),
            version: DB_VERSION,
            compressed_boss_hp,
        };

        let encounter_id = self.insert_encounter(&tx, encounter_db)?;
        let db_entities = Self::to_entities_db(&entities, encounter_id)?;
        self.insert_entities(&tx, encounter_id, db_entities)?;

        let mut players = entities
            .iter()
            .filter(|e| {
                ((e.entity_type == EntityType::Player && e.class_id != 0 && e.max_hp > 0)
                    || e.name == local_player)
                    && e.damage_stats.damage_dealt > 0
            })
            .collect::<Vec<_>>();
        let local_player_dps = players
            .iter()
            .find(|e| e.name == local_player)
            .map(|e| e.damage_stats.dps)
            .unwrap_or_default();
        players.sort_unstable_by_key(|e| Reverse(e.damage_stats.damage_dealt));
        let preview_players = players
            .into_iter()
            .map(|e| format!("{}:{}", e.class_id, e.name))
            .collect::<Vec<_>>()
            .join(",");

        let preview = EncounterPreviewDb {
            encounter_id,
            fight_start: started_on.timestamp_millis(),
            current_boss_name: current_boss_name,
            duration: duration_seconds,
            preview_players,
            raid_difficulty: raid_difficulty.as_ref().to_string(),
            local_player: local_player,
            local_player_dps,
            raid_clear,
            boss_only_damage: boss_only_damage
        };

        self.insert_encounter_preview(&tx, preview)?;

        tx.commit()?;

        Ok(encounter_id)
    }

    pub fn insert_entities(&self, tx: &Transaction, encounter_id: i64, entities: Vec<EntityDb>) -> Result<()> {

        for entity in entities {
            let mut statement = tx.prepare_cached(INSERT_ENTITY)?;

            let sql_params = params![
                entity.name,
                encounter_id,
                entity.npc_id,
                entity.entity_type,
                entity.class_id,
                entity.class,
                entity.gear_score,
                entity.current_hp,
                entity.max_hp,
                entity.is_dead,
                entity.compressed_skills,
                entity.compressed_damage_stats,
                entity.skill_stats,
                entity.dps,
                entity.character_id,
                entity.engraving_data,
                entity.gear_hash,
                entity.ark_passive_active,
                entity.spec,
                entity.ark_passive_data
            ];

            statement.execute(sql_params)?;
        }     

        Ok(())
    }

    pub fn insert_encounter(&self, tx: &Transaction, entity: EncounterDb) -> Result<i64> {
        let mut statement = tx.prepare_cached(INSERT_ENCOUNTER)?;
       
        let sql_params = params![
            entity.last_combat_packet,
            entity.total_damage_dealt,
            entity.top_damage_dealt,
            entity.total_damage_taken,
            entity.top_damage_taken,
            entity.dps,
            entity.compressed_buffs,
            entity.compressed_debuffs,
            entity.total_shielding,
            entity.total_effective_shielding,
            entity.compressed_shields,
            entity.misc,
            entity.version,
            entity.compressed_boss_hp,
        ];

        statement.execute(sql_params)?;

        let id = tx.last_insert_rowid();

        Ok(id)
    }

    pub fn insert_encounter_preview(&self, tx: &Transaction, entity: EncounterPreviewDb) -> Result<()> {
        let mut statement = tx.prepare_cached(INSERT_ENCOUNTER_PREVIEW)?;

        let sql_params = params![
            entity.encounter_id,
            entity.fight_start,
            entity.current_boss_name,
            entity.duration,
            entity.preview_players,
            entity.raid_difficulty,
            entity.local_player,
            entity.local_player_dps,
            entity.raid_clear,
            entity.boss_only_damage
        ];

        statement.execute(sql_params)?;

        Ok(())
    }

    pub fn to_entities_db(entities: &Vec<EncounterEntity>, encounter_id: i64) -> Result<Vec<EntityDb>> {
        let mut entities_db = vec![];

         for entity in entities {

            let compressed_skills = compress_json(&entity.skills)?;
            let compressed_damage_stats = compress_json(&entity.damage_stats)?;

            let entity_db = EntityDb {
                name: entity.name.clone(),
                encounter_id: encounter_id,
                npc_id: entity.npc_id,
                entity_type: entity.entity_type.to_string(),
                class_id: entity.class_id,
                class: entity.class.clone(),
                gear_score: entity.gear_score,
                current_hp: entity.current_hp,
                max_hp: entity.max_hp,
                is_dead: entity.is_dead,
                compressed_skills,
                compressed_damage_stats,
                skill_stats: json!(entity.skill_stats),
                dps: entity.damage_stats.dps,
                character_id: entity.character_id,
                engraving_data: json!(entity.engraving_data),
                gear_hash: entity.gear_hash.clone(),
                ark_passive_active: entity.ark_passive_active,
                spec: entity.spec.clone(),
                ark_passive_data: json!(entity.ark_passive_data)
            };

            entities_db.push(entity_db);;
        }

        Ok(entities_db)
    }

    pub fn save_to_db(model: SaveToDb) {
        
    }
}