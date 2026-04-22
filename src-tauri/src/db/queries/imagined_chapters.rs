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
    /// The step-1 invented scene prose used to drive the illustration.
    /// Stored for debug/replay but intentionally NOT shown to the
    /// chapter writer — the telephone-game inversion is the feature.
    pub scene_description: String,
    /// FK into world_images.image_id. None until step 2 completes.
    pub image_id: Option<String>,
    pub content: String,
    pub created_at: String,
}

pub fn create_imagined_chapter(
    conn: &Connection,
    chapter: &ImaginedChapter,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO imagined_chapters (chapter_id, thread_id, world_day, title, seed_hint, scene_description, image_id, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            chapter.chapter_id,
            chapter.thread_id,
            chapter.world_day,
            chapter.title,
            chapter.seed_hint,
            chapter.scene_description,
            chapter.image_id,
            chapter.content,
            chapter.created_at,
        ],
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
        "SELECT chapter_id, thread_id, world_day, title, seed_hint, scene_description, image_id, content, created_at
         FROM imagined_chapters WHERE chapter_id = ?1",
        params![chapter_id],
        |r| Ok(ImaginedChapter {
            chapter_id: r.get(0)?,
            thread_id: r.get(1)?,
            world_day: r.get(2)?,
            title: r.get(3)?,
            seed_hint: r.get(4)?,
            scene_description: r.get(5)?,
            image_id: r.get(6)?,
            content: r.get(7)?,
            created_at: r.get(8)?,
        }),
    )
}

/// List chapters for a thread, newest first. Used by the modal's sidebar.
pub fn list_imagined_chapters_for_thread(
    conn: &Connection,
    thread_id: &str,
) -> Result<Vec<ImaginedChapter>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT chapter_id, thread_id, world_day, title, seed_hint, scene_description, image_id, content, created_at
         FROM imagined_chapters WHERE thread_id = ?1
         ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![thread_id], |r| Ok(ImaginedChapter {
        chapter_id: r.get(0)?,
        thread_id: r.get(1)?,
        world_day: r.get(2)?,
        title: r.get(3)?,
        seed_hint: r.get(4)?,
        scene_description: r.get(5)?,
        image_id: r.get(6)?,
        content: r.get(7)?,
        created_at: r.get(8)?,
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
