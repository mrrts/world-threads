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
            character_id TEXT REFERENCES characters(character_id) ON DELETE CASCADE,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS messages (
            message_id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system', 'narrative', 'illustration')),
            content TEXT NOT NULL,
            tokens_estimate INTEGER NOT NULL DEFAULT 0,
            sender_character_id TEXT DEFAULT NULL,
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

    // Clean up leftover temp tables from prior failed migrations.
    // Only drop if the real table has data (meaning migration was partially successful).
    conn.execute_batch("DROP TABLE IF EXISTS messages_new;").ok();
    let msgs_ok: i64 = conn.query_row("SELECT count(*) FROM messages", [], |r| r.get(0)).unwrap_or(0);
    if msgs_ok > 0 {
        conn.execute_batch("DROP TABLE IF EXISTS messages_migrating;").ok();
    }
    let gmsgs_ok: i64 = conn.query_row("SELECT count(*) FROM group_messages", [], |r| r.get(0)).unwrap_or(0);
    if gmsgs_ok > 0 {
        conn.execute_batch("DROP TABLE IF EXISTS group_messages_migrating;").ok();
    }

    // Old role migration removed — now handled by the safe CHECK constraint removal below.

    // Fix illustration messages that were stored as JSON {"data_url":"...","caption":"..."}
    // Extract just the data_url value.
    conn.execute_batch("
        UPDATE messages SET content = json_extract(content, '$.data_url')
        WHERE role = 'illustration' AND content LIKE '{%\"data_url\"%';
    ")?;

    // Add video_file column to world_images for illustrations that have been animated
    let has_video_file: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('world_images') WHERE name = 'video_file'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if !has_video_file {
        conn.execute_batch("ALTER TABLE world_images ADD COLUMN video_file TEXT NOT NULL DEFAULT ''")?;
    }

    // Add aspect_ratio column to world_images for layout stability
    let has_aspect_ratio: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('world_images') WHERE name = 'aspect_ratio'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if !has_aspect_ratio {
        conn.execute_batch("ALTER TABLE world_images ADD COLUMN aspect_ratio REAL NOT NULL DEFAULT 0.0")?;
    }

    // Add sender_character_id column to messages (for group chats)
    let has_sender: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('messages') WHERE name = 'sender_character_id'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if !has_sender {
        conn.execute_batch("ALTER TABLE messages ADD COLUMN sender_character_id TEXT DEFAULT NULL")?;
    }

    // Group chats table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS group_chats (
            group_chat_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            character_ids TEXT NOT NULL DEFAULT '[]',
            thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
            display_name TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_group_chats_world ON group_chats(world_id);
    ")?;

    // Make threads.character_id nullable for group chat threads
    // IMPORTANT: disable FK enforcement during table recreation to prevent cascade deletes
    let threads_has_not_null: bool = conn
        .query_row(
            "SELECT sql LIKE '%character_id TEXT NOT NULL%' FROM sqlite_master WHERE type='table' AND name='threads'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if threads_has_not_null {
        conn.execute_batch("
            PRAGMA foreign_keys = OFF;
            CREATE TABLE threads_new (
                thread_id TEXT PRIMARY KEY,
                character_id TEXT REFERENCES characters(character_id) ON DELETE CASCADE,
                world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO threads_new SELECT * FROM threads;
            DROP TABLE threads;
            ALTER TABLE threads_new RENAME TO threads;
            PRAGMA foreign_keys = ON;
        ")?;
    }

    // Separate group_messages table for group chats
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS group_messages (
            message_id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system', 'narrative', 'illustration')),
            content TEXT NOT NULL,
            tokens_estimate INTEGER NOT NULL DEFAULT 0,
            sender_character_id TEXT DEFAULT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_group_messages_thread ON group_messages(thread_id, created_at);

        CREATE VIRTUAL TABLE IF NOT EXISTS group_messages_fts USING fts5(
            message_id UNINDEXED,
            thread_id UNINDEXED,
            content,
            tokenize='porter'
        );
    ")?;

    // Migrate any existing group messages from messages table to group_messages
    let has_group_chats: bool = conn
        .query_row("SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='group_chats'", [], |r| r.get(0))
        .unwrap_or(false);
    if has_group_chats {
        let migrated: i64 = conn.query_row(
            "SELECT count(*) FROM messages m JOIN group_chats gc ON gc.thread_id = m.thread_id",
            [], |r| r.get(0),
        ).unwrap_or(0);
        if migrated > 0 {
            conn.execute_batch("
                INSERT OR IGNORE INTO group_messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at)
                    SELECT m.message_id, m.thread_id, m.role, m.content, m.tokens_estimate, m.sender_character_id, m.created_at
                    FROM messages m
                    JOIN group_chats gc ON gc.thread_id = m.thread_id;

                INSERT OR IGNORE INTO group_messages_fts (message_id, thread_id, content)
                    SELECT m.message_id, m.thread_id, m.content
                    FROM messages m
                    JOIN group_chats gc ON gc.thread_id = m.thread_id
                    WHERE m.role NOT IN ('illustration');

                DELETE FROM messages WHERE thread_id IN (SELECT thread_id FROM group_chats);
            ")?;
        }
    }

    // Purge illustration/video content from FTS indexes (base64 data should never be indexed)
    conn.execute_batch("
        DELETE FROM messages_fts WHERE message_id IN (
            SELECT message_id FROM messages WHERE role IN ('illustration', 'video')
        );
    ").ok();
    conn.execute_batch("
        DELETE FROM group_messages_fts WHERE message_id IN (
            SELECT message_id FROM group_messages WHERE role IN ('illustration', 'video')
        );
    ").ok();

    // Recover any messages stuck in messages_old from a failed migration
    let has_messages_old: bool = conn.prepare("SELECT 1 FROM messages_old LIMIT 0").is_ok();
    if has_messages_old {
        conn.execute_batch("
            INSERT OR IGNORE INTO messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at)
                SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at FROM messages_old;
            DROP TABLE IF EXISTS messages_old;
        ").ok();
    }
    let has_group_messages_old: bool = conn.prepare("SELECT 1 FROM group_messages_old LIMIT 0").is_ok();
    if has_group_messages_old {
        conn.execute_batch("
            INSERT OR IGNORE INTO group_messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at)
                SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at FROM group_messages_old;
            DROP TABLE IF EXISTS group_messages_old;
        ").ok();
    }

    // Safely remove CHECK constraint on role column to allow 'context' role.
    // Only proceed if current table still has the CHECK constraint and messages_old doesn't exist.
    let needs_check_removal: bool = conn.query_row(
        "SELECT sql LIKE '%CHECK%' FROM sqlite_master WHERE type='table' AND name='messages'",
        [], |r| r.get(0),
    ).unwrap_or(false);
    if needs_check_removal {
        let msg_count: i64 = conn.query_row("SELECT count(*) FROM messages", [], |r| r.get(0)).unwrap_or(0);
        // Disable foreign keys for safe table recreation
        conn.execute_batch("PRAGMA foreign_keys=OFF;").ok();
        let result = conn.execute_batch("
            ALTER TABLE messages RENAME TO messages_migrating;
            CREATE TABLE messages (
                message_id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                tokens_estimate INTEGER NOT NULL DEFAULT 0,
                sender_character_id TEXT DEFAULT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                world_day INTEGER DEFAULT NULL,
                world_time TEXT DEFAULT NULL
            );
            INSERT INTO messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time)
                SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM messages_migrating;
        ");
        if result.is_ok() {
            let new_count: i64 = conn.query_row("SELECT count(*) FROM messages", [], |r| r.get(0)).unwrap_or(0);
            if new_count >= msg_count {
                conn.execute_batch("DROP TABLE messages_migrating; CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id, created_at);").ok();
            } else {
                // Rollback: data loss detected
                conn.execute_batch("DROP TABLE messages; ALTER TABLE messages_migrating RENAME TO messages;").ok();
                log::warn!("Messages migration rolled back: count mismatch ({} vs {})", new_count, msg_count);
            }
        } else {
            // Rename back if migration failed
            conn.execute_batch("ALTER TABLE messages_migrating RENAME TO messages;").ok();
        }

        // Same for group_messages
        let gm_count: i64 = conn.query_row("SELECT count(*) FROM group_messages", [], |r| r.get(0)).unwrap_or(0);
        let gresult = conn.execute_batch("
            ALTER TABLE group_messages RENAME TO group_messages_migrating;
            CREATE TABLE group_messages (
                message_id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                tokens_estimate INTEGER NOT NULL DEFAULT 0,
                sender_character_id TEXT DEFAULT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                world_day INTEGER DEFAULT NULL,
                world_time TEXT DEFAULT NULL
            );
            INSERT INTO group_messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time)
                SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM group_messages_migrating;
        ");
        if gresult.is_ok() {
            let new_gm_count: i64 = conn.query_row("SELECT count(*) FROM group_messages", [], |r| r.get(0)).unwrap_or(0);
            if new_gm_count >= gm_count {
                conn.execute_batch("DROP TABLE group_messages_migrating; CREATE INDEX IF NOT EXISTS idx_group_messages_thread ON group_messages(thread_id, created_at);").ok();
            } else {
                conn.execute_batch("DROP TABLE group_messages; ALTER TABLE group_messages_migrating RENAME TO group_messages;").ok();
            }
        } else {
            conn.execute_batch("ALTER TABLE group_messages_migrating RENAME TO group_messages;").ok();
        }
        conn.execute_batch("PRAGMA foreign_keys=ON;").ok();
    }

    // Add world_day and world_time columns to messages tables
    let has_world_day: bool = conn.prepare("SELECT world_day FROM messages LIMIT 0")
        .is_ok();
    if !has_world_day {
        conn.execute_batch("
            ALTER TABLE messages ADD COLUMN world_day INTEGER DEFAULT NULL;
            ALTER TABLE messages ADD COLUMN world_time TEXT DEFAULT NULL;
        ").ok();
        conn.execute_batch("
            ALTER TABLE group_messages ADD COLUMN world_day INTEGER DEFAULT NULL;
            ALTER TABLE group_messages ADD COLUMN world_time TEXT DEFAULT NULL;
        ").ok();
    }

    // ── Novel entries table ──────────────────────────────────────────────
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS novel_entries (
            novel_id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL,
            world_day INTEGER NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(thread_id, world_day)
        );
    ")?;

    // ── Consultant chat tables ──────────────────────────────────────────

    // Migration: if old consultant_messages table exists without chat_id, recreate it
    let old_table_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='consultant_messages'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if old_table_exists {
        let has_chat_id: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('consultant_messages') WHERE name = 'chat_id'",
            [], |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if !has_chat_id {
            // Old schema — rename to preserve data, create new table
            conn.execute("ALTER TABLE consultant_messages RENAME TO consultant_messages_old", []).ok();
        }
    }

    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS consultant_chats (
            chat_id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL,
            title TEXT NOT NULL DEFAULT 'New Chat',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_consultant_chats_thread ON consultant_chats(thread_id);

        CREATE TABLE IF NOT EXISTS consultant_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_consultant_messages_chat ON consultant_messages(chat_id);
    ")?;

    // Remove location from world state JSON (was causing characters to mention town square)
    conn.execute_batch("
        UPDATE worlds SET state = json_remove(state, '$.location') WHERE json_extract(state, '$.location') IS NOT NULL;
    ").ok();

    // Add sex column to characters if missing, default to 'male'
    let has_sex: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('characters') WHERE name = 'sex'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_sex {
        conn.execute("ALTER TABLE characters ADD COLUMN sex TEXT NOT NULL DEFAULT 'male'", []).ok();
    }

    // Add last_seen_message_id to consultant_chats if missing
    let has_last_seen: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('consultant_chats') WHERE name = 'last_seen_message_id'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_last_seen {
        conn.execute("ALTER TABLE consultant_chats ADD COLUMN last_seen_message_id TEXT DEFAULT NULL", []).ok();
    }

    Ok(())
}
