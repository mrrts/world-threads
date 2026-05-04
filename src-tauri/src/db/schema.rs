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

    // user_profiles.boundaries — added 2026-04-25 so the user-character
    // can carry the same boundaries-shape data as regular characters.
    // The canonizer's "Remember this" / "This changes them" flow proposes
    // boundaries for the user when the moment fits; without this column
    // the commit would have nowhere to write them.
    let has_user_boundaries: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('user_profiles') WHERE name = 'boundaries'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !has_user_boundaries {
        conn.execute_batch("ALTER TABLE user_profiles ADD COLUMN boundaries TEXT NOT NULL DEFAULT '[]'")?;
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

    // Addressing: who the speaker is talking to (NULL = unknown; "user" = the
    // human; otherwise a character_id). Additive, nullable — safe on existing
    // rows, which all backfill as NULL.
    let has_address_to: bool = conn.prepare("SELECT address_to FROM messages LIMIT 0").is_ok();
    if !has_address_to {
        conn.execute_batch("ALTER TABLE messages ADD COLUMN address_to TEXT DEFAULT NULL;").ok();
    }
    let has_address_to_group: bool = conn.prepare("SELECT address_to FROM group_messages LIMIT 0").is_ok();
    if !has_address_to_group {
        conn.execute_batch("ALTER TABLE group_messages ADD COLUMN address_to TEXT DEFAULT NULL;").ok();
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

    // Add mode column to consultant_chats: 'immersive' (default — the
    // in-the-story confidant) vs 'backstage' (fourth-wall stage manager
    // that reads the save file). Existing chats default to immersive.
    let has_mode: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('consultant_chats') WHERE name = 'mode'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_mode {
        conn.execute("ALTER TABLE consultant_chats ADD COLUMN mode TEXT NOT NULL DEFAULT 'immersive'", []).ok();
    }

    // Illustration caption column — stores the human-readable text that
    // describes the illustration's subject. Source is either the user's
    // custom_instructions (verbatim) or a "memorable moment" caption that
    // an LLM call picks from recent scene messages when instructions are
    // left blank. Used as alt text + a visible caption in chat views.
    let has_wi_caption: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('world_images') WHERE name = 'caption'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_wi_caption {
        conn.execute("ALTER TABLE world_images ADD COLUMN caption TEXT NOT NULL DEFAULT ''", []).ok();
    }

    // Per-chat current location — replaces the global world.state.location
    // injection (which was a single global location for all chats and
    // leaked nested fields like location.current_scene into every prompt).
    // Each individual chat (threads with character_id) and each group_chat
    // can carry its OWN current_location string. Default NULL = "no
    // location set yet"; UI hides the label until set.
    let has_thread_loc: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('threads') WHERE name = 'current_location'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_thread_loc {
        conn.execute("ALTER TABLE threads ADD COLUMN current_location TEXT DEFAULT NULL", []).ok();
    }
    let has_gc_loc: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('group_chats') WHERE name = 'current_location'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_gc_loc {
        conn.execute("ALTER TABLE group_chats ADD COLUMN current_location TEXT DEFAULT NULL", []).ok();
    }

    // saved_places — per-world place-name library. Populated when the user
    // ticks "save this place" in the location modal. Selecting from the
    // dropdown populates the input. UNIQUE(world_id, name) prevents dup
    // entries; the modal's "save" checkbox is disabled when input matches
    // an existing place to make the constraint unreachable in normal use.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS saved_places (
            saved_place_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(world_id, name)
        );
        CREATE INDEX IF NOT EXISTS idx_saved_places_world ON saved_places(world_id);
    ").ok();

    // Add last_used_at to saved_places — drives 'most-recently-used on top'
    // ordering in the location modal's dropdown. Backfilled to created_at
    // for existing rows; updated on every set_chat_location_cmd call whose
    // new name matches an existing saved place (case-insensitive).
    let has_last_used: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('saved_places') WHERE name = 'last_used_at'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_last_used {
        conn.execute("ALTER TABLE saved_places ADD COLUMN last_used_at TEXT DEFAULT NULL", []).ok();
        conn.execute("UPDATE saved_places SET last_used_at = created_at WHERE last_used_at IS NULL", []).ok();
    }

    // Strip lingering world.state.location.* now that per-chat carries it.
    // The earlier migration only removed top-level `state.location` when
    // it was NULL-leaf; nested objects like {location: {current_scene: ...}}
    // weren't being cleaned. json_remove handles both shapes.
    conn.execute_batch("
        UPDATE worlds SET state = json_remove(state, '$.location') WHERE json_extract(state, '$.location') IS NOT NULL;
    ").ok();

    // Default location: every chat starts in 'Town Square' until the
    // user changes it. Backfill any existing NULL values across both
    // individual chats (threads with character_id NOT NULL) and group
    // chats. New chat creation sites also explicitly write 'Town
    // Square'; the derive_current_location fallback in prompts.rs
    // makes the default visible to the LLM even for brand-new chats
    // that have no location_change messages yet.
    conn.execute("UPDATE threads SET current_location = 'Town Square' WHERE current_location IS NULL AND character_id IS NOT NULL", []).ok();
    conn.execute("UPDATE group_chats SET current_location = 'Town Square' WHERE current_location IS NULL", []).ok();

    // ── Canon entries ─────────────────────────────────────────────────────
    //
    // Records the user's deliberate promotion of a specific message moment
    // into canon. Each row is a historical event: "on date X, message Y was
    // promoted to target Z as canon-type T, with content C."
    //
    // `subject_type` + `subject_id` identify the canon target:
    //   character / <character_id>
    //   user      / <world_id>
    //   world     / <world_id>
    //   relationship / "<char_id_a>::<char_id_b or 'user'>"
    //
    // `record_type`:
    //   description_weave  — revised full description (character or user)
    //                        — THE ONLY ACTIVE MODE. UI exposes nothing else.
    //   known_fact         — DEPRECATED. Constraint kept for historical reads.
    //   relationship_note  — DEPRECATED. Constraint kept for historical reads.
    //   world_fact         — DEPRECATED. Constraint kept for historical reads.
    //
    // The CHECK constraints stay permissive because (per CLAUDE.md DATABASE
    // SAFETY) tightening them would require a recreate+verify migration of
    // a table that already contains user data. The save_kept_record_cmd
    // write path errors out on anything but description_weave, so no new
    // entries with the deprecated record_types can be created.
    //
    // Actual application to the subject row (character.identity updated,
    // user_profile.description updated) happens at the same time; this
    // table is the audit trail + provenance ledger, not the source of
    // truth for the subject. Queryable by source_message_id for "is this
    // message canonized?" indicators.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS kept_records (
            kept_id TEXT PRIMARY KEY,
            source_message_id TEXT,
            source_thread_id TEXT,
            source_world_day INTEGER,
            source_created_at TEXT,
            subject_type TEXT NOT NULL CHECK(subject_type IN ('character','user','world','relationship')),
            subject_id TEXT NOT NULL,
            record_type TEXT NOT NULL CHECK(record_type IN ('description_weave','known_fact','relationship_note','world_fact')),
            content TEXT NOT NULL,
            user_note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_canon_source_message ON kept_records(source_message_id);
        CREATE INDEX IF NOT EXISTS idx_canon_subject ON kept_records(subject_type, subject_id);
    ").ok();

    // ── Reactions: rebuild without the FK to `messages` ─────────────────
    //
    // Original schema declared `message_id TEXT NOT NULL REFERENCES
    // messages(message_id) ON DELETE CASCADE`. A past migration that
    // renamed `messages` caused SQLite to auto-rewrite the FK to point at
    // the renamed table (messages_old / messages_migrating). When that
    // temp table was dropped, the FK became dangling — every INSERT into
    // `reactions` then failed with `no such table: main.messages_old`.
    //
    // We also can't keep the FK anyway now that reactions can target
    // either `messages` or `group_messages` — a foreign key only
    // references one table. Cleanup when messages are deleted is handled
    // in application code.
    //
    // Rebuild pattern follows CLAUDE.md rules: rename → create → copy →
    // verify count → drop or rollback.
    let reactions_sql: Option<String> = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='reactions'",
        [], |r| r.get(0),
    ).ok();
    let reactions_needs_rebuild = reactions_sql
        .as_deref()
        .map_or(false, |s| s.contains("REFERENCES") || s.contains("messages_old") || s.contains("messages_migrating"));

    if reactions_needs_rebuild {
        let count_before: i64 = conn
            .query_row("SELECT count(*) FROM reactions", [], |r| r.get(0))
            .unwrap_or(0);
        conn.execute_batch("PRAGMA foreign_keys=OFF;").ok();
        let rebuild = conn.execute_batch("
            ALTER TABLE reactions RENAME TO reactions_migrating;
            CREATE TABLE reactions (
                reaction_id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                emoji TEXT NOT NULL,
                reactor TEXT NOT NULL CHECK(reactor IN ('user', 'assistant')),
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO reactions (reaction_id, message_id, emoji, reactor, created_at)
                SELECT reaction_id, message_id, emoji, reactor, created_at FROM reactions_migrating;
        ");
        match rebuild {
            Ok(()) => {
                let count_after: i64 = conn
                    .query_row("SELECT count(*) FROM reactions", [], |r| r.get(0))
                    .unwrap_or(0);
                if count_after >= count_before {
                    conn.execute_batch(
                        "DROP TABLE reactions_migrating; CREATE INDEX IF NOT EXISTS idx_reactions_message ON reactions(message_id);"
                    ).ok();
                    log::warn!("reactions table rebuilt: {} rows preserved, FK removed", count_after);
                } else {
                    conn.execute_batch(
                        "DROP TABLE reactions; ALTER TABLE reactions_migrating RENAME TO reactions;"
                    ).ok();
                    log::warn!(
                        "reactions rebuild rolled back: count mismatch ({} vs {})",
                        count_after, count_before
                    );
                }
            }
            Err(e) => {
                conn.execute_batch(
                    "ALTER TABLE reactions_migrating RENAME TO reactions;"
                ).ok();
                log::warn!("reactions rebuild failed, rolled back: {}", e);
            }
        }
        conn.execute_batch("PRAGMA foreign_keys=ON;").ok();
    }

    // Reactions: add sender_character_id for per-character attribution.
    // User reactions keep this as NULL; character-emitted reactions record
    // which character authored them so the UI can surface "Alice reacted
    // 🥺" vs "Bob reacted 🔥" in tooltips.
    let has_sender_char_on_reactions: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('reactions') WHERE name = 'sender_character_id'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_sender_char_on_reactions {
        conn.execute("ALTER TABLE reactions ADD COLUMN sender_character_id TEXT DEFAULT NULL", []).ok();
    }

    // ── Mood reduction (per-thread reaction-emoji ring buffer) ─────────────
    //
    // Feeds back into the AGENCY section: each reaction pushes its emoji
    // onto a JSON array (most-recent-first, capped) that seeds the next
    // mood-note chain. Additive, nullable, safe on existing rows.
    let has_mood_reduction: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('threads') WHERE name = 'mood_reduction'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_mood_reduction {
        conn.execute("ALTER TABLE threads ADD COLUMN mood_reduction TEXT NOT NULL DEFAULT '[]'", []).ok();
    }

    // ── Mood chain persisted per assistant message ────────────────────────
    //
    // The exact 5-emoji chain that was active when this message was
    // generated. Enables the measurement loop: join reactions → messages →
    // mood_chain to surface which chains produce positively-reacted replies.
    // Nullable so existing messages pre-dating the feature stay valid.
    let has_mood_chain_msgs: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name = 'mood_chain'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_mood_chain_msgs {
        conn.execute("ALTER TABLE messages ADD COLUMN mood_chain TEXT DEFAULT NULL", []).ok();
    }
    let has_mood_chain_gmsgs: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('group_messages') WHERE name = 'mood_chain'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_mood_chain_gmsgs {
        conn.execute("ALTER TABLE group_messages ADD COLUMN mood_chain TEXT DEFAULT NULL", []).ok();
    }

    // ── Proactive pings ────────────────────────────────────────────────────
    //
    // Characters can reach out first — unsolicited, between user turns. To
    // keep that from tipping into spam we cap consecutive unanswered pings
    // per thread (reset on any user message) and track when the last ping
    // fired so we can rate-limit across the day.
    //
    // `threads.consecutive_proactive_pings` — incremented each time a ping is
    //    emitted; reset to 0 on any inserted user message. Enforces the
    //    "no more than 2 in a row" rule from the feature spec.
    // `threads.last_proactive_ping_at` — ISO timestamp of the most recent
    //    ping; used for per-thread cooldowns.
    // `messages.is_proactive` — flags a message as a proactive ping so the
    //    UI can style it distinctly and so counters can be replayed.
    let has_cons_pings: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('threads') WHERE name = 'consecutive_proactive_pings'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_cons_pings {
        conn.execute("ALTER TABLE threads ADD COLUMN consecutive_proactive_pings INTEGER NOT NULL DEFAULT 0", []).ok();
    }
    let has_last_ping: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('threads') WHERE name = 'last_proactive_ping_at'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_last_ping {
        conn.execute("ALTER TABLE threads ADD COLUMN last_proactive_ping_at TEXT DEFAULT NULL", []).ok();
    }
    let has_is_proactive: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name = 'is_proactive'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_is_proactive {
        conn.execute("ALTER TABLE messages ADD COLUMN is_proactive INTEGER NOT NULL DEFAULT 0", []).ok();
    }
    // group_messages shares the Message struct and the MSG_COLS constant,
    // so the column must exist on both tables even though proactive pings
    // currently only fire in solo threads. Unused rows stay at default 0.
    let has_is_proactive_g: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('group_messages') WHERE name = 'is_proactive'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_is_proactive_g {
        conn.execute("ALTER TABLE group_messages ADD COLUMN is_proactive INTEGER NOT NULL DEFAULT 0", []).ok();
    }

    // ── Rename kept_records → kept_records ─────────────────────────────────
    //
    // The feature was renamed from "canon" → "kept record" to drop the
    // oversized religious register. Data is preserved: we use SQLite's
    // ALTER TABLE RENAME (table + columns) which is atomic and lossless.
    // CHECK constraints update automatically when columns rename.
    //
    // Runs exactly once: skips if the old table is gone AND the new one
    // already exists. On fresh installs the old table never existed, and
    // the initial CREATE statement further up already uses the new name
    // (see below).
    let has_canon_table: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='kept_records'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    let has_kept_table: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='kept_records'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if has_canon_table && !has_kept_table {
        let before: i64 = conn.query_row("SELECT COUNT(*) FROM kept_records", [], |r| r.get(0)).unwrap_or(-1);
        let rename = conn.execute_batch("
            ALTER TABLE kept_records RENAME TO kept_records;
            ALTER TABLE kept_records RENAME COLUMN kept_id TO kept_id;
            ALTER TABLE kept_records RENAME COLUMN record_type TO record_type;
            DROP INDEX IF EXISTS idx_canon_source_message;
            DROP INDEX IF EXISTS idx_canon_subject;
            CREATE INDEX IF NOT EXISTS idx_kept_source_message ON kept_records(source_message_id);
            CREATE INDEX IF NOT EXISTS idx_kept_subject ON kept_records(subject_type, subject_id);
        ");
        match rename {
            Ok(()) => {
                let after: i64 = conn.query_row("SELECT COUNT(*) FROM kept_records", [], |r| r.get(0)).unwrap_or(-2);
                if after == before {
                    log::warn!("kept_records → kept_records rename succeeded: {} rows preserved", after);
                } else {
                    log::error!("kept_records rename produced count mismatch: {} vs {} — data may be at risk, investigate immediately", before, after);
                }
            }
            Err(e) => {
                log::error!("kept_records → kept_records rename failed: {}. Table is left in its original state; no data lost.", e);
            }
        }
    }
    // Visual description of a character, generated from their active
    // portrait by a multimodal vision call. Lets other characters in
    // the same world know what they look like — used in group-chat and
    // narrative prompts so a character can reference a friend's face
    // the way a real person would. Empty by default; populated on
    // portrait generation/change via an explicit command.
    //
    // `visual_description_portrait_id` caches the portrait_id that
    // generated the current description. When a vision-description
    // refresh is requested, we compare this against the currently-active
    // portrait: if they match, we skip the vision call (the image
    // hasn't changed). Null on legacy rows; set on every successful
    // refresh.
    let has_visual_desc: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('characters') WHERE name = 'visual_description'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_visual_desc {
        conn.execute("ALTER TABLE characters ADD COLUMN visual_description TEXT NOT NULL DEFAULT ''", []).ok();
    }
    let has_visual_desc_src: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('characters') WHERE name = 'visual_description_portrait_id'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_visual_desc_src {
        conn.execute("ALTER TABLE characters ADD COLUMN visual_description_portrait_id TEXT DEFAULT NULL", []).ok();
    }

    // Inventory: per-character "things still in their keeping" (max 3,
    // user-editable, refreshed by LLM on world-day rollover).
    //   inventory: JSON array of { name, description } objects.
    //   last_inventory_day: the world_day index (into World.state.time.day_index)
    //     the inventory was last refreshed against. NULL = never seeded.
    let has_inventory: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('characters') WHERE name = 'inventory'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_inventory {
        conn.execute("ALTER TABLE characters ADD COLUMN inventory TEXT NOT NULL DEFAULT '[]'", []).ok();
    }
    let has_last_inv_day: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('characters') WHERE name = 'last_inventory_day'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_last_inv_day {
        conn.execute("ALTER TABLE characters ADD COLUMN last_inventory_day INTEGER DEFAULT NULL", []).ok();
    }

    // Signature emoji: an optional single emoji the character may
    // occasionally drop into a message when they feel especially
    // themselves in a beat. User-edited from the character settings
    // page. Empty string = disabled (character has no signature).
    let has_signature_emoji: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('characters') WHERE name = 'signature_emoji'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_signature_emoji {
        conn.execute("ALTER TABLE characters ADD COLUMN signature_emoji TEXT NOT NULL DEFAULT ''", []).ok();
    }

    // ── Per-character action-beat density ─────────────────────────────
    //
    // Overrides the global ~1-in-3-replies-no-beat baseline per-character.
    // One of "low" | "normal" | "high". Quiet characters (older, soft-
    // spoken) read better on "low"; alert, in-motion characters on "high".
    // Additive ALTER — safe per the DATABASE SAFETY rule.
    let has_abd: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('characters') WHERE name = 'action_beat_density'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !has_abd {
        conn.execute("ALTER TABLE characters ADD COLUMN action_beat_density TEXT NOT NULL DEFAULT 'normal'", []).ok();
    }

    // One-time clear of existing character inventories so they re-seed
    // under the new mixed physical/interior prompt (soul-level interior
    // items, up to 10 total, kind tag per slot). Any inventories saved
    // before this migration held physical-only items and no kind tag —
    // the next focus refresh on each character will regenerate fresh
    // under the new rules. Gated by a settings marker so it runs once.
    let already_cleared_inv: bool = conn.query_row(
        "SELECT COUNT(*) FROM settings WHERE key = 'schema.inventory_cleared_v6'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !already_cleared_inv {
        let cleared = conn.execute(
            "UPDATE characters SET inventory = '[]', last_inventory_day = NULL WHERE inventory != '[]' OR last_inventory_day IS NOT NULL",
            [],
        ).unwrap_or(0);
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('schema.inventory_cleared_v6', ?1)",
            [chrono::Utc::now().to_rfc3339()],
        ).ok();
        if cleared > 0 {
            log::warn!("Cleared {} existing character inventories for regeneration under the mixed physical/interior prompt", cleared);
        }
    }

    // One-time clear of existing visual descriptions so they regenerate
    // under the tightened vision prompt (no pose/expression/lighting —
    // enduring features only). Gated by a marker row in `settings` so
    // this runs exactly once per database. New descriptions will land
    // via the backfill sweep the next time the app has an API key.
    let already_cleared: bool = conn.query_row(
        "SELECT COUNT(*) FROM settings WHERE key = 'schema.visual_description_cleared_v1'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !already_cleared {
        let cleared = conn.execute(
            "UPDATE characters SET visual_description = '', visual_description_portrait_id = NULL WHERE visual_description != '' OR visual_description_portrait_id IS NOT NULL",
            [],
        ).unwrap_or(0);
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('schema.visual_description_cleared_v1', ?1)",
            [chrono::Utc::now().to_rfc3339()],
        ).ok();
        if cleared > 0 {
            log::warn!("Cleared {} existing visual descriptions for regeneration under the tightened vision prompt", cleared);
        }
    }

    // Fresh-install path: create the new table directly. IF NOT EXISTS means
    // this is a no-op on databases that already came through the rename
    // branch above.
    //
    // record_type CHECK is intentionally broad: the live write types are
    // description_weave / voice_rule / boundary / known_fact / open_loop
    // (the auto-classifier can emit any of these). Deprecated legacy
    // values relationship_note and world_fact stay in the constraint so
    // pre-existing rows from the old flow remain readable — they are
    // NEVER emitted by the write path.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS kept_records (
            kept_id TEXT PRIMARY KEY,
            source_message_id TEXT,
            source_thread_id TEXT,
            source_world_day INTEGER,
            source_created_at TEXT,
            subject_type TEXT NOT NULL CHECK(subject_type IN ('character','user','world','relationship')),
            subject_id TEXT NOT NULL,
            record_type TEXT NOT NULL CHECK(record_type IN ('description_weave','voice_rule','boundary','known_fact','open_loop','relationship_note','world_fact')),
            content TEXT NOT NULL,
            user_note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_kept_source_message ON kept_records(source_message_id);
        CREATE INDEX IF NOT EXISTS idx_kept_subject ON kept_records(subject_type, subject_id);
    ").ok();

    // ── One-shot migration: broaden kept_records.record_type CHECK ────────
    //
    // Existing installs have a narrower CHECK that only admits
    // description_weave / known_fact / relationship_note / world_fact.
    // The auto-canonization classifier also emits voice_rule / boundary /
    // open_loop, so we need to rebuild the table with the broader
    // constraint. Idempotent via the `schema.kept_records_check_v2`
    // setting marker.
    //
    // Safety protocol (per CLAUDE.md DATABASE SAFETY rule):
    //   1. Rename old table → kept_records_migrating
    //   2. Create new table with broader CHECK
    //   3. INSERT all rows from migrating → new
    //   4. VERIFY row count matches; if not, ROLLBACK (rename back)
    //   5. Only drop the migrating table on verified success
    let already_migrated: i64 = conn.query_row(
        "SELECT COUNT(*) FROM settings WHERE key = 'schema.kept_records_check_v2'",
        [], |r| r.get(0),
    ).unwrap_or(0);
    if already_migrated == 0 {
        let table_exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='kept_records'",
            [], |r| r.get(0),
        ).unwrap_or(0);
        if table_exists > 0 {
            let before: i64 = conn.query_row("SELECT COUNT(*) FROM kept_records", [], |r| r.get(0)).unwrap_or(-1);
            let migrate_result: rusqlite::Result<()> = (|| {
                conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
                conn.execute_batch("ALTER TABLE kept_records RENAME TO kept_records_migrating;")?;
                conn.execute_batch("
                    CREATE TABLE kept_records (
                        kept_id TEXT PRIMARY KEY,
                        source_message_id TEXT,
                        source_thread_id TEXT,
                        source_world_day INTEGER,
                        source_created_at TEXT,
                        subject_type TEXT NOT NULL CHECK(subject_type IN ('character','user','world','relationship')),
                        subject_id TEXT NOT NULL,
                        record_type TEXT NOT NULL CHECK(record_type IN ('description_weave','voice_rule','boundary','known_fact','open_loop','relationship_note','world_fact')),
                        content TEXT NOT NULL,
                        user_note TEXT NOT NULL DEFAULT '',
                        created_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );
                ")?;
                conn.execute_batch("
                    INSERT INTO kept_records
                        (kept_id, source_message_id, source_thread_id, source_world_day,
                         source_created_at, subject_type, subject_id, record_type,
                         content, user_note, created_at)
                    SELECT kept_id, source_message_id, source_thread_id, source_world_day,
                           source_created_at, subject_type, subject_id, record_type,
                           content, user_note, created_at
                    FROM kept_records_migrating;
                ")?;
                Ok(())
            })();
            match migrate_result {
                Ok(()) => {
                    let after: i64 = conn.query_row("SELECT COUNT(*) FROM kept_records", [], |r| r.get(0)).unwrap_or(-2);
                    if after == before {
                        // Verified — safe to drop the old table and mark migrated.
                        conn.execute_batch("
                            DROP TABLE kept_records_migrating;
                            CREATE INDEX IF NOT EXISTS idx_kept_source_message ON kept_records(source_message_id);
                            CREATE INDEX IF NOT EXISTS idx_kept_subject ON kept_records(subject_type, subject_id);
                            PRAGMA foreign_keys=ON;
                        ").ok();
                        conn.execute(
                            "INSERT INTO settings (key, value) VALUES ('schema.kept_records_check_v2', ?1)",
                            [chrono::Utc::now().to_rfc3339()],
                        ).ok();
                        log::warn!("kept_records CHECK broadened to v2: {} rows preserved", after);
                    } else {
                        // Count mismatch — ROLLBACK.
                        conn.execute_batch("
                            DROP TABLE IF EXISTS kept_records;
                            ALTER TABLE kept_records_migrating RENAME TO kept_records;
                            PRAGMA foreign_keys=ON;
                        ").ok();
                        log::error!("kept_records CHECK migration count mismatch ({} vs {}) — rolled back, original table restored. No data lost.", before, after);
                    }
                }
                Err(e) => {
                    // Migration failed partway through — restore original.
                    conn.execute_batch("
                        DROP TABLE IF EXISTS kept_records;
                        ALTER TABLE kept_records_migrating RENAME TO kept_records;
                        PRAGMA foreign_keys=ON;
                    ").ok();
                    log::error!("kept_records CHECK migration failed: {}. Table is left in its original state; no data lost.", e);
                }
            }
        } else {
            // Fresh install — no migration needed; just mark v2 to skip the
            // check next launch.
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('schema.kept_records_check_v2', ?1)",
                [chrono::Utc::now().to_rfc3339()],
            ).ok();
        }
    }

    // ── Character inventory snapshots ─────────────────────────────────────
    //
    // Time-travel ledger for per-character inventory state. Written BEFORE
    // every inventory mutation (seed, refresh, moment-update, user edit)
    // so each row represents "what the inventory WAS just prior to this
    // change". Used by `reset_to_message_cmd` to rewind a character's
    // keeping alongside the messages when the user resets to a point in
    // history — the world and the thing-in-hand stay in sync.
    //
    // Capped at ~50 rows per character via trim-on-insert. The trigger
    // column is diagnostic: "seed" | "refresh" | "moment" | "user_edit".
    // FK cascade on character deletion cleans up automatically.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS character_inventory_snapshots (
            snapshot_id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            inventory TEXT NOT NULL DEFAULT '[]',
            last_inventory_day INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            trigger TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_inv_snapshots_char_time
            ON character_inventory_snapshots(character_id, created_at DESC);
    ").ok();

    // ── Inventory update records ──────────────────────────────────────────
    //
    // One row per (message_id, character_id) pair recording the diff from
    // the moment-anchored inventory update that was triggered from that
    // message. Used to render an inline "Inventory updated ✓: ..."
    // badge under messages that have produced an update, and to persist
    // that indicator across app restarts.
    //
    // `added` / `updated` / `removed` are JSON arrays of item names
    // (strings). Empty arrays are valid — a row with all-empty arrays
    // means "the moment was considered but no change was made" (only
    // possible on the narrative-in-group pure-maintain path). Writing
    // such rows would clutter the UI, so callers only insert when at
    // least one of the three arrays is non-empty.
    //
    // PK (message_id, character_id) with ON CONFLICT REPLACE semantics
    // means re-running the button overwrites the prior record for that
    // character — each message shows the latest update's diff, not a
    // history. No FK on message_id (messages live in two tables);
    // character_id FK cascades cleanup on character deletion.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS inventory_update_records (
            message_id TEXT NOT NULL,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            added TEXT NOT NULL DEFAULT '[]',
            updated TEXT NOT NULL DEFAULT '[]',
            removed TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (message_id, character_id)
        );
        CREATE INDEX IF NOT EXISTS idx_inv_upd_message ON inventory_update_records(message_id);
    ").ok();

    // One-shot backfill for inventory_update messages written before the
    // dispatcher started copying world_day / world_time from world state.
    // Without this, the frontend's TimeDivider sees a NULL-time meta row
    // between two same-time messages and falsely draws a new-time divider
    // on the next real message the user sends. We pick the nearest real
    // message in the same thread (prior preferred, next as fallback) and
    // copy its fields onto the inventory_update row. Gated by a settings
    // marker so the sweep runs exactly once.
    let inv_wt_backfilled: bool = conn.query_row(
        "SELECT COUNT(*) FROM settings WHERE key = 'schema.inventory_update_world_time_backfilled_v1'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !inv_wt_backfilled {
        let patched_solo = conn.execute("
            UPDATE messages
            SET world_day = COALESCE(
                    (SELECT m2.world_day FROM messages m2
                     WHERE m2.thread_id = messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at <= messages.created_at
                       AND m2.world_day IS NOT NULL
                     ORDER BY m2.created_at DESC LIMIT 1),
                    (SELECT m2.world_day FROM messages m2
                     WHERE m2.thread_id = messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at > messages.created_at
                       AND m2.world_day IS NOT NULL
                     ORDER BY m2.created_at ASC LIMIT 1)
                ),
                world_time = COALESCE(
                    (SELECT m2.world_time FROM messages m2
                     WHERE m2.thread_id = messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at <= messages.created_at
                       AND m2.world_time IS NOT NULL
                     ORDER BY m2.created_at DESC LIMIT 1),
                    (SELECT m2.world_time FROM messages m2
                     WHERE m2.thread_id = messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at > messages.created_at
                       AND m2.world_time IS NOT NULL
                     ORDER BY m2.created_at ASC LIMIT 1)
                )
            WHERE role = 'inventory_update' AND (world_day IS NULL OR world_time IS NULL)
        ", []).unwrap_or(0);
        let patched_group = conn.execute("
            UPDATE group_messages
            SET world_day = COALESCE(
                    (SELECT m2.world_day FROM group_messages m2
                     WHERE m2.thread_id = group_messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at <= group_messages.created_at
                       AND m2.world_day IS NOT NULL
                     ORDER BY m2.created_at DESC LIMIT 1),
                    (SELECT m2.world_day FROM group_messages m2
                     WHERE m2.thread_id = group_messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at > group_messages.created_at
                       AND m2.world_day IS NOT NULL
                     ORDER BY m2.created_at ASC LIMIT 1)
                ),
                world_time = COALESCE(
                    (SELECT m2.world_time FROM group_messages m2
                     WHERE m2.thread_id = group_messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at <= group_messages.created_at
                       AND m2.world_time IS NOT NULL
                     ORDER BY m2.created_at DESC LIMIT 1),
                    (SELECT m2.world_time FROM group_messages m2
                     WHERE m2.thread_id = group_messages.thread_id
                       AND m2.role <> 'inventory_update'
                       AND m2.created_at > group_messages.created_at
                       AND m2.world_time IS NOT NULL
                     ORDER BY m2.created_at ASC LIMIT 1)
                )
            WHERE role = 'inventory_update' AND (world_day IS NULL OR world_time IS NULL)
        ", []).unwrap_or(0);
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('schema.inventory_update_world_time_backfilled_v1', ?1)",
            [chrono::Utc::now().to_rfc3339()],
        ).ok();
        if patched_solo > 0 || patched_group > 0 {
            log::warn!(
                "[Schema] backfilled world_day/world_time on {} solo + {} group inventory_update rows",
                patched_solo, patched_group,
            );
        }
    }

    // ── Character journals ────────────────────────────────────────────────
    //
    // One first-person reflective entry per character per world-day.
    // Generated by a cheap memory_model call that takes the character's
    // identity, current inventory (physical + interior), today's
    // messages involving them, and the previous 1-2 entries. Written in
    // the character's own voice — not a recap, a reflection.
    //
    // Fed back into dialogue prompts as "who you've been lately" so the
    // LLM reads its own account of itself and stays coherent over long
    // spans. Visible to the user in the CharacterEditor so the journal
    // is also a readable artifact.
    //
    // UNIQUE(character_id, world_day) — one per day per character; if
    // the user regenerates, ON CONFLICT REPLACE overwrites. FK cascade
    // on character_id cleans up if a character is ever deleted.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS character_journals (
            journal_id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            world_day INTEGER NOT NULL DEFAULT 0,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(character_id, world_day)
        );
        CREATE INDEX IF NOT EXISTS idx_journals_char_day
            ON character_journals(character_id, world_day DESC);
    ").ok();

    // ── User (player) journals ────────────────────────────────────────────
    //
    // Parallel to character_journals but keyed by world_id — one entry
    // per world per world-day, written as the player-character
    // retrospecting yesterday across every chat they were in.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS user_journals (
            journal_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            world_day INTEGER NOT NULL DEFAULT 0,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(world_id, world_day)
        );
        CREATE INDEX IF NOT EXISTS idx_user_journals_world_day
            ON user_journals(world_id, world_day DESC);
    ").ok();

    // ── Quests ───────────────────────────────────────────────────────────
    //
    // User-accepted pursuits for a world. The model (via Backstage)
    // proposes; the user ratifies by accepting. One row per quest. Active
    // quests have completed_at = NULL; completion is a user act (or a
    // Backstage proposal the user ratifies). Intentionally NOT a quest-
    // system-with-mechanics: no deadlines, no progress bars, no auto-
    // completion. Active quests show up in the dialogue-prompt world
    // context so characters know them implicitly; they surface in
    // dialogue only when the moment is right.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS quests (
            quest_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            title TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            accepted_at TEXT NOT NULL DEFAULT (datetime('now')),
            accepted_world_day INTEGER,
            completed_at TEXT,
            completed_world_day INTEGER,
            completion_note TEXT NOT NULL DEFAULT '',
            abandoned_at TEXT,
            abandoned_world_day INTEGER,
            abandonment_note TEXT NOT NULL DEFAULT '',
            origin_kind TEXT NOT NULL DEFAULT 'user_authored',
            origin_ref TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_quests_world_active
            ON quests(world_id, completed_at, abandoned_at);
    ").ok();

    // Non-destructive ADD COLUMN migrations for databases created under
    // an earlier shape of the quests table.
    let cols: Vec<(&str, &str)> = vec![
        ("abandoned_at", "TEXT"),
        ("abandoned_world_day", "INTEGER"),
        ("abandonment_note", "TEXT NOT NULL DEFAULT ''"),
        ("origin_kind", "TEXT NOT NULL DEFAULT 'user_authored'"),
        ("origin_ref", "TEXT"),
    ];
    for (name, ty) in cols {
        let has: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('quests') WHERE name = ?1",
            rusqlite::params![name], |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if !has {
            conn.execute(&format!("ALTER TABLE quests ADD COLUMN {name} {ty}"), []).ok();
        }
    }

    // One-shot wipe of existing journal entries so every character
    // re-generates under the new world-day-bounded logic (history feed
    // strictly today's messages; prior entries strictly before today).
    // Gated by a settings marker so this runs exactly once per database.
    let already_wiped_journals: bool = conn.query_row(
        "SELECT COUNT(*) FROM settings WHERE key = 'schema.journals_cleared_v1'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !already_wiped_journals {
        let wiped = conn.execute("DELETE FROM character_journals", []).unwrap_or(0);
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('schema.journals_cleared_v1', ?1)",
            [chrono::Utc::now().to_rfc3339()],
        ).ok();
        if wiped > 0 {
            log::warn!("[Schema] cleared {wiped} journal entries for regeneration under bounded-history logic");
        }
    }

    // ── Daily readings ────────────────────────────────────────────────────
    //
    // One qualitative "reading" per world per world-day — a field-report
    // card across fixed craft domains (agape, daylight, soundness,
    // aliveness, honesty, undercurrents) plus a single "poignant
    // complication" that names what's still open at the end of the day.
    // Generated by a two-pass memory_model chain: first-pass draft,
    // then a self-critique that refines for honesty / specificity /
    // not-performative.
    //
    // UNIQUE(world_id, world_day) — one per day per world; regenerate
    // via ON CONFLICT REPLACE. Domains stored as JSON so the axis list
    // can evolve without schema churn.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS daily_readings (
            reading_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            world_day INTEGER NOT NULL DEFAULT 0,
            domains TEXT NOT NULL DEFAULT '[]',
            complication TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(world_id, world_day)
        );
        CREATE INDEX IF NOT EXISTS idx_daily_readings_world_day
            ON daily_readings(world_id, world_day DESC);
    ").ok();

    // ── Meanwhile events ─────────────────────────────────────────────────
    //
    // Small off-screen texture: one or two lines per character per
    // generation cycle describing what they were doing when the user
    // wasn't around. Not plot, not stakes — texture. The goal is that
    // opening the app feels like arriving somewhere real instead of
    // resuming a chatbot.
    //
    // Generated on demand (a "Generate meanwhile" button in the
    // sidebar) or on world-day rollover. Each row is one compact event
    // belonging to one character at a specific day + time-of-day.
    // Shown in the sidebar's World State area as a short scrollable
    // feed.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS meanwhile_events (
            event_id TEXT PRIMARY KEY,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            world_day INTEGER NOT NULL DEFAULT 0,
            time_of_day TEXT NOT NULL DEFAULT '',
            summary TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_meanwhile_world_time
            ON meanwhile_events(world_id, created_at DESC);
    ").ok();

    // imagined_chapters — novel chapters of moments that DIDN'T happen
    // in chat but are plausible-in-world / in-character. The "Imagined
    // Chapter" feature runs a three-stage telephone pipeline per chapter:
    //   1. Invent a specific visual moment for the chat's characters
    //      (optional user hint).
    //   2. Render that description as an illustration with character +
    //      user portraits as reference images.
    //   3. Feed ONLY the image + reference portraits into a vision-aware
    //      model and stream a novel chapter that answers the image.
    // The step-1 scene_description is stored for debug/replay but NOT
    // shown to the chapter writer — the inversion ("image-first, prose
    // answers") is the whole point.
    //
    // Scoped to thread_id so the sidebar of past chapters for a given
    // chat mirrors the consultant-chats sidebar pattern. image_id
    // references world_images.image_id so the illustration lives
    // alongside the rest of the gallery and gets the same storage +
    // backup treatment.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS imagined_chapters (
            chapter_id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL,
            world_day INTEGER,
            title TEXT NOT NULL DEFAULT '',
            seed_hint TEXT NOT NULL DEFAULT '',
            scene_location TEXT DEFAULT NULL,
            scene_description TEXT NOT NULL DEFAULT '',
            image_id TEXT,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_imagined_chapters_thread
            ON imagined_chapters(thread_id, created_at DESC);
    ").ok();

    // breadcrumb_message_id — links a chapter to the role='imagined_chapter'
    // row inserted into the chat transcript when the chapter was saved.
    // Lets the canon flow look up the breadcrumb by chapter_id (the existing
    // canon classifier expects a source_message_id; this column is what the
    // frontend hands over for "canonize this chapter"). Added via
    // ALTER TABLE ADD COLUMN per CLAUDE.md DATABASE SAFETY rule.
    let column_exists: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('imagined_chapters') WHERE name = 'breadcrumb_message_id'",
        [], |_| Ok(true),
    ).unwrap_or(false);
    if !column_exists {
        let _ = conn.execute("ALTER TABLE imagined_chapters ADD COLUMN breadcrumb_message_id TEXT", []);
    }

    // canonized — chapters now exist in TWO states: pre-canon (just an
    // entry in the modal sidebar, no chat-history footprint) and
    // canonized (chapter "counts" — breadcrumb row inserted into the
    // chat transcript so the model + user see it as part of history).
    // Only canonized chapters reach the dialogue prompt's history block.
    // ALTER TABLE ADD COLUMN per CLAUDE.md DATABASE SAFETY rule.
    //
    // Backfill: any pre-existing chapter that has a breadcrumb_message_id
    // is treated as canonized (matches current observable state — the
    // breadcrumbs are already in the chat). New rows default to 0.
    let canonized_col_exists: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('imagined_chapters') WHERE name = 'canonized'",
        [], |_| Ok(true),
    ).unwrap_or(false);
    if !canonized_col_exists {
        let _ = conn.execute("ALTER TABLE imagined_chapters ADD COLUMN canonized INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute(
            "UPDATE imagined_chapters SET canonized = 1 WHERE breadcrumb_message_id IS NOT NULL",
            [],
        );
    }

    let scene_location_col_exists: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('imagined_chapters') WHERE name = 'scene_location'",
        [],
        |_| Ok(true),
    ).unwrap_or(false);
    if !scene_location_col_exists {
        conn.execute(
            "ALTER TABLE imagined_chapters ADD COLUMN scene_location TEXT DEFAULT NULL",
            [],
        )?;
    }

    // ── dev_chat_sessions / dev_chat_messages ─────────────────────────────
    //
    // Out-of-band conversations between the developer (Claude Code, via
    // the `worldcli` binary) and the user's characters. These are
    // INVISIBLE to the UI — no chat in the app reads or displays them.
    // Used for craft-extraction work: Claude Code asks a character a
    // meta question (per the "ask the character" pattern in CLAUDE.md),
    // mines the answer for craft material, ships into prompts.rs.
    //
    // Schema is intentionally minimal — name + role + content + time.
    // No reactions, no portraits, no inventory tracking. Ephemeral
    // working memory for prompt-stack development.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS dev_chat_sessions (
            session_id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            character_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS dev_chat_messages (
            message_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('user','assistant')),
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES dev_chat_sessions(session_id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_dev_chat_messages_session ON dev_chat_messages(session_id);
    ").ok();

    // ─── relational_stances ────────────────────────────────────────────
    //
    // Per-character synthesized prose, written in the character's own
    // voice, capturing how they've come to see the user RIGHT NOW.
    // Append-only history (one row per refresh); reads always select the
    // most recent. Never surfaced to the player — pure prompt-stack
    // background warmth that lets characters be measurably more attuned
    // over time without exposing a meter or score.
    //
    // Generated by an LLM pass that synthesizes kept_records + recent
    // journals + a sample of recent exchanges. Triggered:
    //   1) after canonization commits (the two-act gate has just fired —
    //      a load-bearing moment is what made this worth re-synthesizing)
    //   2) on the first dialogue turn of a new in-world day (a natural
    //      cadence boundary; previous stance was last day's read of them)
    //
    // Both triggers fire-and-forget so the user's reply is never blocked;
    // the in-flight turn uses whatever stance exists, the next benefits.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS relational_stances (
            stance_id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            stance_text TEXT NOT NULL,
            world_day_at_generation INTEGER,
            source_kept_record_count INTEGER NOT NULL DEFAULT 0,
            source_journal_count INTEGER NOT NULL DEFAULT 0,
            source_message_count INTEGER NOT NULL DEFAULT 0,
            refresh_trigger TEXT NOT NULL,
            model_used TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_relational_stances_char
            ON relational_stances(character_id, created_at DESC);
    ").ok();

    // ── character_load_test_anchors ─────────────────────────────────────
    //
    // Per-character "what does this character load-test?" — the
    // architecture-level spine of their authority, periodically synthesized
    // from their recent corpus. Direct sibling of `relational_stances` in
    // both schema shape and refresh pattern. See 2026-04-24-0948 report
    // for the experiment that confirmed explicit anchor-naming shifts
    // character-specific behavior.
    //
    // anchor_label: short identifier like "DEVOTION" / "LANGUAGE" /
    //   "FABRIC OF A LIFE" — the dimension the character weight-tests.
    // anchor_body: the full second-person prompt-block injected into
    //   the dialogue system prompt. Self-contained, ready to push into
    //   the parts vec without further formatting.
    // derivation_summary: one paragraph explaining how the anchor was
    //   derived from the corpus. Inspectable; helps the user see WHY
    //   this character got this anchor.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS character_load_test_anchors (
            anchor_id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL REFERENCES characters(character_id) ON DELETE CASCADE,
            world_id TEXT NOT NULL REFERENCES worlds(world_id) ON DELETE CASCADE,
            anchor_label TEXT NOT NULL,
            anchor_body TEXT NOT NULL,
            derivation_summary TEXT NOT NULL DEFAULT '',
            world_day_at_generation INTEGER,
            source_message_count INTEGER NOT NULL DEFAULT 0,
            refresh_trigger TEXT NOT NULL,
            model_used TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_load_test_anchors_char
            ON character_load_test_anchors(character_id, created_at DESC);
    ").ok();

    // Multi-axis pivot: add axis_kind column so the same table can hold
    // multiple register-axes per character (load_test, joy_reception,
    // grief, ...). Backwards-compatible: existing rows default to
    // 'load_test'. ALTER TABLE ADD COLUMN with NOT NULL needs a default
    // for the migration to succeed; we use 'load_test' as the floor.
    let has_axis_kind: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('character_load_test_anchors') WHERE name = 'axis_kind'",
        [],
        |_| Ok(true),
    ).unwrap_or(false);
    if !has_axis_kind {
        conn.execute_batch("
            ALTER TABLE character_load_test_anchors
                ADD COLUMN axis_kind TEXT NOT NULL DEFAULT 'load_test';
            CREATE INDEX IF NOT EXISTS idx_load_test_anchors_char_axis
                ON character_load_test_anchors(character_id, axis_kind, created_at DESC);
        ").ok();
    }

    // ── derived_formula columns on worlds, characters, user_profiles ─────
    //
    // Documentary-form derived-formula storage. Each entity (world,
    // character, user) gets an optional `derived_formula TEXT` column
    // holding a character-canonical formula-shorthand derivation of F
    // = (R, C) for that entity.
    //
    // Per the design discipline shipped at .claude/memory/feedback_
    // auto_derivation_design_discipline.md and the cross-world arc
    // (reports/2026-04-26-0815/0829/0832/0845), derivations are NOT
    // injected at the dialogue layer (substrate-swap empirically null).
    // They are stored for documentary use:
    //   - Backstage Consultant reads them as additional context
    //   - reports/ and persona-sims cite them
    //   - open-source forks find them as readable shorthand
    //
    // Population: deferred. Initial values are NULL. worldcli derive-
    // world / derive-character commands populate manually; AI-trigger-
    // on-save is a v2 question Ryan named as "let it sit a beat."
    //
    // ALTER TABLE ADD COLUMN per CLAUDE.md DATABASE SAFETY rule.
    let world_has_derivation: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('worlds') WHERE name = 'derived_formula'",
        [], |_| Ok(true),
    ).unwrap_or(false);
    if !world_has_derivation {
        let _ = conn.execute("ALTER TABLE worlds ADD COLUMN derived_formula TEXT", []);
    }
    let char_has_derivation: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('characters') WHERE name = 'derived_formula'",
        [], |_| Ok(true),
    ).unwrap_or(false);
    if !char_has_derivation {
        let _ = conn.execute("ALTER TABLE characters ADD COLUMN derived_formula TEXT", []);
    }
    let user_has_derivation: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('user_profiles') WHERE name = 'derived_formula'",
        [], |_| Ok(true),
    ).unwrap_or(false);
    if !user_has_derivation {
        let _ = conn.execute("ALTER TABLE user_profiles ADD COLUMN derived_formula TEXT", []);
    }

    // ── derived_formula_updated_at — auto-derivation cadence pipeline ──
    //
    // Per the auto-derivation pipeline shipped 2026-04-26 ~21:00 (design
    // consult at /tmp/derivation-design-response.json, synthesis module
    // src/ai/derivation.rs). Hybrid OR staleness policy: refresh when
    // either time threshold OR event-count threshold is hit. NULL = stale
    // (never derived). ALTER TABLE ADD COLUMN per CLAUDE.md DATABASE
    // SAFETY rule.
    for table in &["worlds", "characters", "user_profiles"] {
        let has_col: bool = conn.query_row(
            &format!("SELECT 1 FROM pragma_table_info('{table}') WHERE name = 'derived_formula_updated_at'"),
            [], |_| Ok(true),
        ).unwrap_or(false);
        if !has_col {
            let _ = conn.execute(
                &format!("ALTER TABLE {table} ADD COLUMN derived_formula_updated_at TEXT"),
                [],
            );
        }
    }

    // ── derived_summary — friendly-prose companion to derived_formula ──
    //
    // The derived_formula field stores Unicode-math derivation for cast-listing
    // injection (load-bearing for prompt-stack); derived_summary stores a
    // human-readable plain-English translation of the same derivation for
    // UI display. Two-output synthesis pipeline produces both from one
    // ChatGPT call when invoked via the Maggie-friendly UI flow (per the
    // 5-sub-question wizard at frontend/src/components/UserProfileEditor.tsx).
    //
    // ALTER TABLE ADD COLUMN per CLAUDE.md DATABASE SAFETY rule.
    for table in &["worlds", "characters", "user_profiles"] {
        let has_col: bool = conn.query_row(
            &format!("SELECT 1 FROM pragma_table_info('{table}') WHERE name = 'derived_summary'"),
            [], |_| Ok(true),
        ).unwrap_or(false);
        if !has_col {
            let _ = conn.execute(
                &format!("ALTER TABLE {table} ADD COLUMN derived_summary TEXT"),
                [],
            );
        }
    }

    // ── formula_signature on messages + group_messages ──────────────────
    //
    // Per-assistant-message Formula momentstamp (the chat-state signature
    // computed against 𝓕 := (𝓡, 𝓒) when the user has chosen reactions=off).
    // Stored INLINE on the assistant message that the signature was used
    // to condition. Two purposes:
    //
    //   1. Visibility in chat histories sent to LLMs. Chat-history
    //      formatters render the signature as an inline prefix on
    //      assistant messages with one, so downstream LLMs (conscience
    //      grader, memory updater, reaction picker, etc.) see the
    //      momentstamp natively when reading the message stream.
    //
    //   2. Stateful chain. When computing the next momentstamp, read
    //      the LATEST formula_signature from prior assistant messages
    //      in this chat and pass it to ChatGPT as prior_signature
    //      context. The signature evolves as a hash-chain-style running
    //      cumulation rather than being recomputed from scratch each
    //      turn.
    //
    // ALTER TABLE ADD COLUMN per CLAUDE.md DATABASE SAFETY rule.
    // Nullable, no default — most messages will not have a signature
    // (only assistant replies generated under reactions=off, after the
    // momentstamp module shipped).
    let messages_have_sig: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('messages') WHERE name = 'formula_signature'",
        [], |_| Ok(true),
    ).unwrap_or(false);
    if !messages_have_sig {
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN formula_signature TEXT", []);
    }
    let group_messages_have_sig: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('group_messages') WHERE name = 'formula_signature'",
        [], |_| Ok(true),
    ).unwrap_or(false);
    if !group_messages_have_sig {
        let _ = conn.execute("ALTER TABLE group_messages ADD COLUMN formula_signature TEXT", []);
    }

    // ── has_read_empiricon on characters ─────────────────────────────────
    //
    // When true, the full Empiricon report text is injected into this
    // character's LLM prompts (dialogue, dreams, narration, novelization,
    // formula derivation, momentstamp). In-universe: the character has
    // read the document and shares that substrate with the human.
    let char_has_empiricon: bool = conn.query_row(
        "SELECT 1 FROM pragma_table_info('characters') WHERE name = 'has_read_empiricon'",
        [],
        |_| Ok(true),
    )
    .unwrap_or(false);
    if !char_has_empiricon {
        let _ = conn.execute(
            "ALTER TABLE characters ADD COLUMN has_read_empiricon INTEGER NOT NULL DEFAULT 0",
            [],
        );
    }

    // ── location_derivations cache ───────────────────────────────────────
    //
    // Per-(world, location-name) cached derivation of how this location
    // instantiates 𝓒 within 𝓕 = (𝓡, 𝓒). Locations are free-form strings
    // on threads.current_location / group_chats.current_location, with
    // an optional saved_places library for naming reuse. The derivation
    // table is keyed by (world_id, name COLLATE NOCASE) so the same
    // location name reuses its derivation across solo and group chats
    // and across any number of threads in the same world.
    //
    // Generated by ai::derivation::derive_location on first use (when
    // set_chat_location_cmd updates a chat to a name not yet in the
    // table for this world) via background tokio task; reads are
    // synchronous at prompt-build time.
    //
    // Same shape as worlds.derived_formula and characters.derived_formula
    // — feeds the elevation injection in orchestrator::run_dialogue_with_base
    // when CHARACTER_FORMULA_AT_TOP=1, placed between the world block
    // and the character block (zoom-from-world: world → location →
    // character → moment).
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS location_derivations (
            world_id TEXT NOT NULL,
            name TEXT NOT NULL,
            derived_formula TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (world_id, name COLLATE NOCASE),
            FOREIGN KEY (world_id) REFERENCES worlds(world_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_location_derivations_world ON location_derivations(world_id);"
    )?;

    Ok(())
}
