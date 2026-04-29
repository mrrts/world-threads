use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// One imagined chapter (invented scene → illustration → vision-written prose).
/// Scoped per-thread so chapters live alongside the chat that inspired them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImaginedChapter {
    pub chapter_id: String,
    pub thread_id: String,
    pub world_day: Option<i64>,
    pub title: String,
    pub seed_hint: String,
    pub scene_location: Option<String>,
    /// The step-1 invented scene prose used to drive the illustration.
    /// Stored for debug/replay but intentionally NOT shown to the
    /// chapter writer — the telephone-game inversion is the feature.
    pub scene_description: String,
    /// FK into world_images.image_id. None until step 2 completes.
    pub image_id: Option<String>,
    pub content: String,
    pub created_at: String,
    /// FK into messages.message_id (or group_messages) — the chat-history
    /// breadcrumb row inserted when this chapter was canonized. Null
    /// for non-canonized chapters (no chat-history footprint).
    pub breadcrumb_message_id: Option<String>,
    /// Whether this chapter has been blessed into canon. Pre-canon
    /// chapters live in the modal sidebar but don't appear in chat
    /// history and don't reach the dialogue prompt's history block.
    pub canonized: bool,
}

pub fn create_imagined_chapter(
    conn: &Connection,
    chapter: &ImaginedChapter,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO imagined_chapters (chapter_id, thread_id, world_day, title, seed_hint, scene_location, scene_description, image_id, content, created_at, breadcrumb_message_id, canonized)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            chapter.chapter_id,
            chapter.thread_id,
            chapter.world_day,
            chapter.title,
            chapter.seed_hint,
            chapter.scene_location,
            chapter.scene_description,
            chapter.image_id,
            chapter.content,
            chapter.created_at,
            chapter.breadcrumb_message_id,
            chapter.canonized as i32,
        ],
    )?;
    Ok(())
}

/// Set the canonized flag. Called by canonize_imagined_chapter_cmd
/// after inserting the breadcrumb message; never set directly by the
/// generation flow.
pub fn set_imagined_chapter_canonized(
    conn: &Connection,
    chapter_id: &str,
    canonized: bool,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE imagined_chapters SET canonized = ?2 WHERE chapter_id = ?1",
        params![chapter_id, canonized as i32],
    )?;
    Ok(())
}

/// Set the breadcrumb_message_id on an existing chapter row. Called
/// once after the chat-history breadcrumb is inserted so the canon flow
/// can resolve "canonize this chapter" → an actual source_message_id.
pub fn set_imagined_chapter_breadcrumb(
    conn: &Connection,
    chapter_id: &str,
    breadcrumb_message_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE imagined_chapters SET breadcrumb_message_id = ?2 WHERE chapter_id = ?1",
        params![chapter_id, breadcrumb_message_id],
    )?;
    Ok(())
}

/// Update the content / title on an existing chapter. Used by the
/// streaming command to write the final chapter text once the stream
/// completes (the row is created early so the frontend can reference
/// its id during streaming).
pub fn update_imagined_chapter(
    conn: &Connection,
    chapter_id: &str,
    title: &str,
    content: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE imagined_chapters SET title = ?2, content = ?3 WHERE chapter_id = ?1",
        params![chapter_id, title, content],
    )?;
    Ok(())
}

pub fn set_imagined_chapter_image(
    conn: &Connection,
    chapter_id: &str,
    image_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE imagined_chapters SET image_id = ?2 WHERE chapter_id = ?1",
        params![chapter_id, image_id],
    )?;
    Ok(())
}

pub fn get_imagined_chapter(
    conn: &Connection,
    chapter_id: &str,
) -> Result<ImaginedChapter, rusqlite::Error> {
    conn.query_row(
        "SELECT chapter_id, thread_id, world_day, title, seed_hint, scene_location, scene_description, image_id, content, created_at, breadcrumb_message_id, canonized
         FROM imagined_chapters WHERE chapter_id = ?1",
        params![chapter_id],
        |r| Ok(ImaginedChapter {
            chapter_id: r.get(0)?,
            thread_id: r.get(1)?,
            world_day: r.get(2)?,
            title: r.get(3)?,
            seed_hint: r.get(4)?,
            scene_location: r.get(5)?,
            scene_description: r.get(6)?,
            image_id: r.get(7)?,
            content: r.get(8)?,
            created_at: r.get(9)?,
            breadcrumb_message_id: r.get(10)?,
            canonized: r.get::<_, i32>(11)? != 0,
        }),
    )
}

/// List chapters for a thread, newest first. Used by the modal's sidebar.
pub fn list_imagined_chapters_for_thread(
    conn: &Connection,
    thread_id: &str,
) -> Result<Vec<ImaginedChapter>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT chapter_id, thread_id, world_day, title, seed_hint, scene_location, scene_description, image_id, content, created_at, breadcrumb_message_id, canonized
         FROM imagined_chapters WHERE thread_id = ?1
         ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![thread_id], |r| Ok(ImaginedChapter {
        chapter_id: r.get(0)?,
        thread_id: r.get(1)?,
        world_day: r.get(2)?,
        title: r.get(3)?,
        seed_hint: r.get(4)?,
        scene_location: r.get(5)?,
        scene_description: r.get(6)?,
        image_id: r.get(7)?,
        content: r.get(8)?,
        created_at: r.get(9)?,
        breadcrumb_message_id: r.get(10)?,
        canonized: r.get::<_, i32>(11)? != 0,
    }))?;
    rows.collect()
}

pub fn delete_imagined_chapter(
    conn: &Connection,
    chapter_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM imagined_chapters WHERE chapter_id = ?1",
        params![chapter_id],
    )?;
    Ok(())
}

pub fn rename_imagined_chapter(
    conn: &Connection,
    chapter_id: &str,
    title: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE imagined_chapters SET title = ?2 WHERE chapter_id = ?1",
        params![chapter_id, title],
    )?;
    Ok(())
}

pub fn set_imagined_chapter_scene_location(
    conn: &Connection,
    chapter_id: &str,
    scene_location: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE imagined_chapters SET scene_location = ?2 WHERE chapter_id = ?1",
        params![chapter_id, scene_location],
    )?;
    Ok(())
}
