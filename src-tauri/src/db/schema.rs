use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS worlds (
            world_id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT 'Untitled World',
            description TEXT NOT NULL DEFAULT '',
            tone_tags TEXT NOT NULL DEFAULT '[]',
            invariants TEXT NOT NULL DEFAULT '[]',
            state TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS user_profiles (
            world_id TEXT PRIMARY KEY REFERENCES worlds(world_id) ON DELETE CASCADE,
            display_name TEXT NOT NULL DEFAULT 'Me',
            description TEXT NOT NULL DEFAULT '',
            facts TEXT NOT NULL DEFAULT '[]',
            avatar_file TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS characters (
            character_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            display_name TEXT NOT NULL,
            identity TEXT NOT NULL DEFAULT '',
            voice_rules TEXT NOT NULL DEFAULT '[]',
            boundaries TEXT NOT NULL DEFAULT '[]',
            backstory_facts TEXT NOT NULL DEFAULT '[]',
            relationships TEXT NOT NULL DEFAULT '{}',
            state TEXT NOT NULL DEFAULT '{}',
            avatar_color TEXT NOT NULL DEFAULT '#c4a882',
            is_archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS threads (
            thread_id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS messages (
            message_id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system', 'narrative')),
            content TEXT NOT NULL,
            tokens_estimate INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id, created_at);

        CREATE TABLE IF NOT EXISTS world_events (
            event_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            day_index INTEGER NOT NULL DEFAULT 0,
            time_of_day TEXT NOT NULL DEFAULT 'MORNING',
            summary TEXT NOT NULL,
            involved_characters TEXT NOT NULL DEFAULT '[]',
            hooks TEXT NOT NULL DEFAULT '[]',
            trigger_type TEXT NOT NULL DEFAULT 'after_user_message',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_world_events_world ON world_events(world_id, created_at);

        CREATE TABLE IF NOT EXISTS memory_artifacts (
            artifact_id TEXT PRIMARY KEY,
            artifact_type TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            sources TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_memory_subject ON memory_artifacts(subject_id, artifact_type);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tick_cache (
            cache_key TEXT PRIMARY KEY,
            result TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS message_count_tracker (
            thread_id TEXT PRIMARY KEY,
            count_since_maintenance INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS character_portraits (
            portrait_id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            prompt TEXT NOT NULL,
            file_name TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_portraits_character ON character_portraits(character_id);

        CREATE TABLE IF NOT EXISTS world_images (
            image_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            prompt TEXT NOT NULL,
            file_name TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0,
            source TEXT NOT NULL DEFAULT 'generated',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_world_images_world ON world_images(world_id);

        CREATE TABLE IF NOT EXISTS chat_backgrounds (
            character_id TEXT PRIMARY KEY REFERENCES characters(character_id) ON DELETE CASCADE,
            bg_type TEXT NOT NULL DEFAULT 'color',
            bg_color TEXT NOT NULL DEFAULT '',
            bg_image_id TEXT NOT NULL DEFAULT '',
            bg_blur INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS token_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            call_type TEXT NOT NULL,
            model TEXT NOT NULL,
            prompt_tokens INTEGER NOT NULL DEFAULT 0,
            completion_tokens INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_token_usage_date ON token_usage(created_at);

        CREATE TABLE IF NOT EXISTS reactions (
            reaction_id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL REFERENCES messages(message_id) ON DELETE CASCADE,
            emoji TEXT NOT NULL,
            reactor TEXT NOT NULL CHECK(reactor IN ('user', 'assistant')),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_reactions_message ON reactions(message_id);

        CREATE TABLE IF NOT EXISTS character_mood (
            character_id TEXT PRIMARY KEY REFERENCES characters(character_id) ON DELETE CASCADE,
            valence REAL NOT NULL DEFAULT 0.0,
            energy REAL NOT NULL DEFAULT 0.0,
            tension REAL NOT NULL DEFAULT 0.0,
            history TEXT NOT NULL DEFAULT '[]',
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ")?;

    // Migrate old content-synced FTS tables to standalone ones.
    // Old schema had content_rowid=rowid which doesn't work with TEXT PKs.
    let needs_fts_migration: bool = conn.query_row(
        "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='messages_fts' AND sql LIKE '%content_rowid%'",
        [], |r| r.get(0),
    ).unwrap_or(false);

    if needs_fts_migration {
        conn.execute_batch("
            DROP TABLE IF EXISTS messages_fts;
            DROP TABLE IF EXISTS world_events_fts;
        ")?;
    }

    conn.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            message_id UNINDEXED,
            thread_id UNINDEXED,
            content,
            tokenize='porter'
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS world_events_fts USING fts5(
            event_id UNINDEXED,
            world_id UNINDEXED,
            summary,
            tokenize='porter'
        );
    ")?;

    // If we just migrated, backfill FTS from existing data
    if needs_fts_migration {
        conn.execute_batch("
            INSERT INTO messages_fts (message_id, thread_id, content)
            SELECT message_id, thread_id, content FROM messages;

            INSERT INTO world_events_fts (event_id, world_id, summary)
            SELECT event_id, world_id, summary FROM world_events;
        ")?;
    }

    let has_vec: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='vec_chunks'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !has_vec {
        conn.execute_batch("
            CREATE VIRTUAL TABLE IF NOT EXISTS vec_chunks USING vec0(
                embedding float[1536]
            );

            CREATE TABLE IF NOT EXISTS chunk_metadata (
                rowid INTEGER PRIMARY KEY,
                chunk_id TEXT NOT NULL UNIQUE,
                source_type TEXT NOT NULL,
                source_id TEXT NOT NULL,
                world_id TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
        ")?;
    }

    // Add is_archived column to characters if missing (migration for existing DBs)
    let has_archived: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('characters') WHERE name = 'is_archived'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !has_archived {
        conn.execute_batch("ALTER TABLE characters ADD COLUMN is_archived INTEGER NOT NULL DEFAULT 0")?;
    }

    let has_bg_image_id: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('chat_backgrounds') WHERE name = 'bg_image_id'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !has_bg_image_id {
        conn.execute_batch("ALTER TABLE chat_backgrounds ADD COLUMN bg_image_id TEXT NOT NULL DEFAULT ''")?;
    }

    let has_avatar_file: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('user_profiles') WHERE name = 'avatar_file'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !has_avatar_file {
        conn.execute_batch("ALTER TABLE user_profiles ADD COLUMN avatar_file TEXT NOT NULL DEFAULT ''")?;
    }

    let has_source: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('world_images') WHERE name = 'source'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !has_source {
        conn.execute_batch("ALTER TABLE world_images ADD COLUMN source TEXT NOT NULL DEFAULT 'generated'")?;
    }

    // Scrub "Studio Ghibli" / "Miyazaki" references from stored prompts
    conn.execute_batch("
        UPDATE world_images SET prompt = REPLACE(REPLACE(REPLACE(prompt,
            'Studio Ghibli watercolor landscape painting. ', 'Watercolor landscape painting. '),
            'like a panoramic frame from a Miyazaki film', 'wide panoramic composition'),
            'Studio Ghibli ', '');
        UPDATE character_portraits SET prompt = REPLACE(REPLACE(REPLACE(prompt,
            'Studio Ghibli watercolor portrait of a character.', 'Watercolor portrait of a character.'),
            'like a frame from a Miyazaki film', 'dreamy atmosphere'),
            'Studio Ghibli ', '');
    ")?;

    // Gallery archive & tags columns
    let has_wi_archived: bool = conn
        .query_row("SELECT count(*) > 0 FROM pragma_table_info('world_images') WHERE name = 'is_archived'", [], |r| r.get(0))
        .unwrap_or(false);
    if !has_wi_archived {
        conn.execute_batch("
            ALTER TABLE world_images ADD COLUMN is_archived INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE world_images ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
        ")?;
    }

    let has_cp_archived: bool = conn
        .query_row("SELECT count(*) > 0 FROM pragma_table_info('character_portraits') WHERE name = 'is_archived'", [], |r| r.get(0))
        .unwrap_or(false);
    if !has_cp_archived {
        conn.execute_batch("
            ALTER TABLE character_portraits ADD COLUMN is_archived INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE character_portraits ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
        ")?;
    }

    // Add character_id to chunk_metadata for per-character vector search.
    // Existing chunks lack this data, so wipe and let them regenerate.
    let has_chunk_char_id: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('chunk_metadata') WHERE name = 'character_id'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !has_chunk_char_id {
        // Wipe existing vectors — they'll be regenerated with character_id on next chat
        conn.execute_batch("
            DELETE FROM vec_chunks;
            DELETE FROM chunk_metadata;
        ")?;
        conn.execute_batch(
            "ALTER TABLE chunk_metadata ADD COLUMN character_id TEXT NOT NULL DEFAULT ''"
        )?;
    }

    // Add 'narrative' to the messages.role CHECK constraint.
    // SQLite doesn't support ALTER CHECK, so we detect the old constraint and recreate.
    let role_check_old: bool = conn
        .query_row(
            "SELECT sql LIKE '%role IN (''user'', ''assistant'', ''system'')%' FROM sqlite_master WHERE type='table' AND name='messages'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if role_check_old {
        conn.execute_batch("
            CREATE TABLE messages_new (
                message_id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
                role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system', 'narrative')),
                content TEXT NOT NULL,
                tokens_estimate INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO messages_new SELECT * FROM messages;
            DROP TABLE messages;
            ALTER TABLE messages_new RENAME TO messages;
            CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id, created_at);
        ")?;

        // Rebuild FTS to match
        conn.execute_batch("
            DELETE FROM messages_fts;
            INSERT INTO messages_fts (message_id, thread_id, content)
            SELECT message_id, thread_id, content FROM messages;
        ")?;
    }

    Ok(())
}
