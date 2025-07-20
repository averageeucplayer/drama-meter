CREATE TABLE Migration (
    file_name NVARCHAR(50) NOT NULL PRIMARY KEY,
    recorded_on INTEGER NOT NULL
);

CREATE TABLE Player (
    character_id INTEGER NOT NULL PRIMARY KEY,
    name NVARCHAR(50) NOT NULL,
    class_id INTEGER NOT NULL,
    gear_score REAL NOT NULL,
    created_on INTEGER NOT NULL,
    updated_on INTEGER NOT NULL
);

CREATE INDEX IX_Players_name ON Player(name);

CREATE TABLE Raid (
    id SMALLINT PRIMARY KEY,
    created_on INTEGER NOT NULL,
    name NVARCHAR(50) NOT NULL,
    zone_name NVARCHAR(50) NOT NULL,
    gate TINYINT NULL, 
    npc_ids BLOB NOT NULL
);

CREATE TABLE Player_stats (
    character_id INTEGER NOT NULL PRIMARY KEY,
    raid_id INTEGER NOT NULL,
    encounter_id INTEGER NOT NULL,
    created_on INTEGER NOT NULL,
    duration INTEGER NOT NULL,
    total_damage INTEGER NOT NULL,
    dps INTEGER NOT NULL,
    brand_uptime REAL NOT NULL,
    attack_power_uptime REAL NOT NULL,
    identity_uptime REAL NOT NULL
);

CREATE TABLE Damage_log (
    encounter_id INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    value BLOB,
    PRIMARY KEY (encounter_id, character_id)
);