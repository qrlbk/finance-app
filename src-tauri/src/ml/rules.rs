//! Rule-based category lookup: normalized note -> category_id.
//! Used before ML prediction for exact matches (hybrid rules + ML).

use rusqlite::Connection;

const MIN_NOTE_LEN: usize = 3;
const MAX_NOTE_LEN: usize = 200;
const MAX_RULES: usize = 500;

/// Normalize note for rule storage and lookup: trim, lowercase, collapse whitespace.
pub fn normalize_note(note: &str) -> String {
    let s = note.trim().to_lowercase();
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Check if note length is valid for rules (3..=200).
pub fn is_note_length_ok(note: &str) -> bool {
    let n = note.trim().len();
    n >= MIN_NOTE_LEN && n <= MAX_NOTE_LEN
}

/// Lookup category by normalized note for the given user. Returns (category_id, category_name) or None.
pub fn lookup(conn: &Connection, user_id: i64, note: &str) -> Option<(i64, String)> {
    let key = normalize_note(note);
    if key.len() < MIN_NOTE_LEN || key.len() > MAX_NOTE_LEN {
        return None;
    }
    let mut stmt = conn
        .prepare(
            "SELECT r.category_id, c.name FROM category_rules r
             JOIN categories c ON c.id = r.category_id AND c.user_id = r.user_id
             WHERE r.user_id = ?1 AND r.note_normalized = ?2",
        )
        .ok()?;
    let mut rows = stmt.query(rusqlite::params![user_id, &key]).ok()?;
    let row = rows.next().ok()??;
    let category_id: i64 = row.get(0).ok()?;
    let category_name: String = row.get(1).ok()?;
    Some((category_id, category_name))
}

/// Upsert rule: note_normalized -> category_id for the given user. Enforces MAX_RULES per user.
pub fn upsert_rule(conn: &Connection, user_id: i64, note: &str, category_id: i64) -> Result<(), String> {
    let key = normalize_note(note);
    if key.len() < MIN_NOTE_LEN || key.len() > MAX_NOTE_LEN {
        return Ok(());
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM category_rules WHERE user_id = ?1", [user_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    if count >= MAX_RULES as i64 {
        conn.execute(
            "DELETE FROM category_rules WHERE user_id = ?1 AND id IN (
                SELECT id FROM category_rules WHERE user_id = ?1 ORDER BY created_at ASC LIMIT 1
            )",
            rusqlite::params![user_id, user_id],
        )
        .map_err(|e| e.to_string())?;
    }

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "DELETE FROM category_rules WHERE user_id = ?1 AND note_normalized = ?2",
        rusqlite::params![user_id, &key],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO category_rules (user_id, note_normalized, category_id, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![user_id, &key, category_id, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_note() {
        assert_eq!(normalize_note("  Glovo  доставка  "), "glovo доставка");
        assert_eq!(normalize_note("МАГАЗИН"), "магазин");
    }

    #[test]
    fn test_is_note_length_ok() {
        assert!(!is_note_length_ok("ab"));
        assert!(is_note_length_ok("abc"));
        assert!(is_note_length_ok(&"a".repeat(200)));
        assert!(!is_note_length_ok(&"a".repeat(201)));
    }
}
