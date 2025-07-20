
pub const DELETE_FROM_ENCOUNTERS: &'static str = r#"DELETE FROM encounter"#;

pub const DELETE_FROM_ENCOUNTERS_IDS: &'static str = r#"DELETE FROM encounter WHERE id IN ({})"#;

pub const DELETE_FROM_ENCOUNTERS_UNCLEARED: &'static str = r#"
    DELETE
    FROM encounter
    WHERE id IN (
        SELECT id
        FROM encounter_preview
        WHERE cleared = 0
)"#;

pub const DELETE_FROM_ENCOUNTERS_UNCLEARED_KEEP_FAVOURITE: &'static str = r#"
    DELET
    FROM encounter
    WHERE id IN (
        SELECT id
        FROM encounter_preview
        WHERE cleared = 0 AND favorite = 0
    )
"#;

pub const DELETE_FROM_ENCOUNTERS_INCLUDE_PREVIEW_IDS: &'static str = r#"
    DELETE
    FROM encounter
    WHERE id IN (
        SELECT id
        FROM encounter_preview
        WHERE favorite = 0
    )"#;

pub const DELETE_FROM_ENCOUNTERS_ID: &'static str = r#"
    DELETE
    FROM encounter
    WHERE id = ?;
"#;

pub const DELETE_FROM_ENCOUNTERS_BELOW_DURATION_KEEP_FAVOURITE: &'static str = r#"
    DELETE
    FROM encounter
    WHERE id IN (
        SELECT id
        FROM encounter_preview
        WHERE duration < ? AND favorite = 0
)"#;

pub const DELETE_FROM_ENCOUNTERS_BELOW_DURATION: &'static str = r#"
    DELETE
    FROM encounter
    WHERE id IN (
        SELECT id
        FROM encounter_preview
        WHERE duration < ?
)"#;

pub const INSERT_SYNC_LOG: &'static str = r#"
    INSERT OR REPLACE INTO sync_logs
    (encounter_id, upstream_id, failed)
    VALUES
    (?, ?, ?);
"#;

pub const GET_LATEST_ENCOUNTER: &'static str = r#"
    SELECT id
    FROM encounter_preview
    ORDER BY fight_start DESC
    LIMIT 1;
"#;

pub const UPDATE_ENCOUNTER_FAVOURITE: &'static str = r#"
    UPDATE encounter_preview
    SET favorite = NOT favorite
    WHERE id = ?;
"#;

pub const ENCOUNTER_SEARCH_INSERT: &'static str = r#"
    INSERT INTO encounter_search
    (encounter_search)
    VALUES('optimize');
    VACUUM;
"#;

pub const SELECT_ENCOUNTER_JOIN_PREVIEW_BY_ID: &'static str = r#"
    SELECT
        last_combat_packet,
        fight_start,
        local_player,
        current_boss,
        duration,
        total_damage_dealt,
        top_damage_dealt,
        total_damage_taken,
        top_damage_taken,
        dps,
        buffs,
        debuffs,
        misc,
        difficulty,
        favorite,
        cleared,
        boss_only_damage,
        total_shielding,
        total_effective_shielding,
        applied_shield_buffs,
        boss_hp_log
    FROM encounter
    JOIN encounter_preview
    USING (id)
    WHERE id = ?
"#;

pub const INSERT_ENCOUNTER_PREVIEW: &'static str = r#"
  INSERT INTO encounter_preview (
        id,
        fight_start,
        current_boss,
        duration,
        players,
        difficulty,
        local_player,
        my_dps,
        cleared,
        boss_only_damage
    )
    VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
"#;

pub const INSERT_ENCOUNTER: &'static str = r#"
    INSERT INTO encounter (
        last_combat_packet,
        total_damage_dealt,
        top_damage_dealt,
        total_damage_taken,
        top_damage_taken,
        dps,
        buffs,
        debuffs,
        total_shielding,
        total_effective_shielding,
        applied_shield_buffs,
        misc,
        version,
        boss_hp_log
    )
    VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
"#;

pub const INSERT_ENTITY: &'static str = r#"
    INSERT INTO entity (
        name,
        encounter_id,
        npc_id,
        entity_type,
        class_id,
        class,
        gear_score,
        current_hp,
        max_hp,
        is_dead,
        skills,
        damage_stats,
        skill_stats,
        dps,
        character_id,
        engravings,
        gear_hash,
        ark_passive_active,
        spec,
        ark_passive_data
    )
    VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
"#;

pub const SELECT_ENTITY_BY_ENCOUNTER_ID: &'static str = r#"
    SELECT
        name,
        class_id,
        class,
        gear_score,
        current_hp,
        max_hp,
        is_dead,
        skills,
        damage_stats,
        skill_stats,
        last_update,
        entity_type,
        npc_id,
        character_id,
        engravings,
        spec,
        ark_passive_active,
        ark_passive_data
    FROM entity
    WHERE encounter_id = ?;
"#;

pub const SELECT_SYNC_LOG_BY_ENCOUNTER_ID: &'static str = r#"
    SELECT
        upstream_id
    FROM sync_logs
    WHERE encounter_id = ?
        AND failed = false;"#;

pub const SELECT_ENCOUNTER_PREVIEW: &'static str = r#"
let query = format!(
    SELECT
        e.id,
        e.fight_start,
        e.current_boss,
        e.duration,
        e.difficulty,
        e.favorite,
        e.cleared,
        e.local_player,
        e.my_dps,
        e.players
    FROM encounter_preview e {join}
    WHERE e.duration > ? {boss}
    {clear}
    {favorite}
    {difficulty}
    {boss_only_damage}
    ORDER BY {sort} {order}
    LIMIT ?
    OFFSET ?",
"#;

pub const SELECT_STATS: &'static str = r#"
    SELECT
        (SELECT COUNT(*) FROM encounter_preview) encounter_count,
        (SELECT COUNT(*) FROM encounter_preview WHERE duration >= ?) encounter_filtered_count
"#;

pub const SELECT_ENCOUNTER_PREVIEW_COUNT: &'static str = r#"SELECT COUNT(*) FROM encounter_preview"#;

pub const SELECT_ENCOUNTER_PREVIEW_FILTERED_COUNT: &'static str = r#"
    SELECT
        COUNT(*)
    FROM encounter_preview e {join}
    WHERE duration > ? {}
    {} {clear} {favorite} {boss_only_damage}
"#;

pub const SELECT_LATEST_ENCOUNTER_ID: &'static str = "
    SELECT
        id
    FROM encounter_preview
    ORDER BY fight_start DESC
    LIMIT 1;
";

/// https://www.sqlite.org/fts5.html
pub const INSERT_FTS5: &'static str = r#"
    INSERT INTO encounter_search
    (encounter_search)
    VALUES('optimize');
    VACUUM;
"#;

pub const SELECT_ENCOUNTER_PREVIEW_JOIN_SYNC_FILTERED: &str = r#"
    SELECT id
    FROM encounter_preview
    LEFT JOIN sync_logs
        ON encounter_id = id
    WHERE cleared = true
        AND boss_only_damage = 1
        AND upstream_id {}
    ORDER BY fight_start;
"#;