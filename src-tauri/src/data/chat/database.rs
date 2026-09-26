// src-tauri/src/data/chat/database.rs
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: i64,
    pub title: String,
    pub model_name: String,
    /// JSON-encoded `SessionSettings`; `None` for legacy rows.
    pub settings: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionWithMessages {
    pub session: ChatSession,
    pub messages: Vec<ChatMessage>,
}

#[derive(Clone)]
pub struct ChatDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl ChatDatabase {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, String> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?;

        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

        let db_path = app_dir.join("chat_history.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

        Self::initialize_database(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn initialize_database(conn: &Connection) -> Result<(), String> {
        // SQLite disables foreign keys per-connection by default, so the
        // ON DELETE CASCADE below is inert unless we opt in.
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT DEFAULT 'New Chat',
                model_name TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                thinking TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Add columns introduced after the initial schema.
        if !Self::column_exists(conn, "chat_messages", "thinking")? {
            conn.execute("ALTER TABLE chat_messages ADD COLUMN thinking TEXT", [])
                .map_err(|e| e.to_string())?;
        }
        if !Self::column_exists(conn, "chat_sessions", "settings")? {
            conn.execute("ALTER TABLE chat_sessions ADD COLUMN settings TEXT", [])
                .map_err(|e| e.to_string())?;
        }

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session ON chat_messages(session_id)",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_updated ON chat_sessions(updated_at)",
            [],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let name: String = row.get(1).map_err(|e| e.to_string())?;
            if name == column {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // Session operations
    pub async fn create_session(
        &self,
        model_name: &str,
        title: Option<&str>,
    ) -> Result<i64, String> {
        let title = title.unwrap_or("New Chat");
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO chat_sessions (title, model_name) VALUES (?1, ?2)",
            params![title, model_name],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    pub async fn get_sessions(&self) -> Result<Vec<ChatSession>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, model_name, settings, created_at, updated_at 
             FROM chat_sessions 
             ORDER BY updated_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ChatSession {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    model_name: row.get(2)?,
                    settings: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row.map_err(|e| e.to_string())?);
        }
        Ok(sessions)
    }

    pub async fn get_session_with_messages(
        &self,
        session_id: i64,
    ) -> Result<Option<ChatSessionWithMessages>, String> {
        // Get the session first
        let session = {
            let conn = self.conn.lock().await;
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, model_name, settings, created_at, updated_at 
                 FROM chat_sessions 
                 WHERE id = ?1",
                )
                .map_err(|e| e.to_string())?;

            let mut rows = stmt.query(params![session_id]).map_err(|e| e.to_string())?;

            if let Some(row) = rows.next().map_err(|e| e.to_string())? {
                Some(ChatSession {
                    id: row.get(0).map_err(|e| e.to_string())?,
                    title: row.get(1).map_err(|e| e.to_string())?,
                    model_name: row.get(2).map_err(|e| e.to_string())?,
                    settings: row.get(3).map_err(|e| e.to_string())?,
                    created_at: row.get(4).map_err(|e| e.to_string())?,
                    updated_at: row.get(5).map_err(|e| e.to_string())?,
                })
            } else {
                None
            }
        }; // All rusqlite types are dropped here before the await

        // Now get messages if we have a session
        if let Some(session) = session {
            let messages = self.get_messages_for_session(session_id).await?;
            Ok(Some(ChatSessionWithMessages { session, messages }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_session_title(&self, session_id: i64, title: &str) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE chat_sessions SET title = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![title, session_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Persist the JSON-encoded session settings.
    pub async fn update_session_settings(
        &self,
        session_id: i64,
        settings_json: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE chat_sessions SET settings = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![settings_json, session_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn delete_session(&self, session_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM chat_sessions WHERE id = ?1",
            params![session_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Message operations
    pub async fn add_message(
        &self,
        session_id: i64,
        role: &str,
        content: &str,
        thinking: Option<&str>,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        // Update session updated_at
        conn.execute(
            "UPDATE chat_sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![session_id],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO chat_messages (session_id, role, content, thinking) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, role, content, thinking],
        ).map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    pub async fn get_messages_for_session(
        &self,
        session_id: i64,
    ) -> Result<Vec<ChatMessage>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, role, content, thinking, created_at 
             FROM chat_messages 
             WHERE session_id = ?1 
             ORDER BY id ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    thinking: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row.map_err(|e| e.to_string())?);
        }
        Ok(messages)
    }

    pub async fn get_messages_for_session_since(
        &self,
        session_id: i64,
        since: i64,
    ) -> Result<Vec<ChatMessage>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, role, content, thinking, created_at 
             FROM chat_messages 
             WHERE session_id = ?1 AND id > ?2
             ORDER BY id ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![session_id, since], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    thinking: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row.map_err(|e| e.to_string())?);
        }
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> ChatDatabase {
        let conn = Connection::open_in_memory().unwrap();
        ChatDatabase::initialize_database(&conn).unwrap();
        ChatDatabase {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    #[tokio::test]
    async fn session_crud_roundtrip() {
        let db = test_db();

        let id = db.create_session("llama3:latest", None).await.unwrap();
        let sessions = db.get_sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, id);
        assert_eq!(sessions[0].title, "New Chat");
        assert_eq!(sessions[0].model_name, "llama3:latest");

        db.update_session_title(id, "Renamed").await.unwrap();
        let sessions = db.get_sessions().await.unwrap();
        assert_eq!(sessions[0].title, "Renamed");

        db.delete_session(id).await.unwrap();
        assert!(db.get_sessions().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn stores_messages_with_thinking() {
        let db = test_db();
        let id = db.create_session("m", Some("Chat")).await.unwrap();

        db.add_message(id, "user", "hi", None).await.unwrap();
        db.add_message(id, "assistant", "hello", Some("reasoning"))
            .await
            .unwrap();

        let messages = db.get_messages_for_session(id).await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "hi");
        assert_eq!(messages[0].thinking, None);
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "hello");
        assert_eq!(messages[1].thinking.as_deref(), Some("reasoning"));
    }

    #[tokio::test]
    async fn messages_are_returned_in_insertion_order() {
        let db = test_db();
        let id = db.create_session("m", Some("Chat")).await.unwrap();

        // Inserted in the same second, so created_at ties; id is the tiebreak.
        for (role, content) in [("user", "one"), ("assistant", "two"), ("user", "three")] {
            db.add_message(id, role, content, None).await.unwrap();
        }

        let messages = db.get_messages_for_session(id).await.unwrap();
        let contents: Vec<&str> = messages.iter().map(|m| m.content.as_str()).collect();
        assert_eq!(contents, ["one", "two", "three"]);
    }

    #[tokio::test]
    async fn get_messages_since_excludes_older_ids() {
        let db = test_db();
        let id = db.create_session("m", None).await.unwrap();
        let first = db.add_message(id, "user", "one", None).await.unwrap();
        db.add_message(id, "assistant", "two", None).await.unwrap();

        let since = db.get_messages_for_session_since(id, first).await.unwrap();
        assert_eq!(since.len(), 1);
        assert_eq!(since[0].content, "two");
    }

    #[tokio::test]
    async fn get_session_with_messages_returns_none_for_missing() {
        let db = test_db();
        assert!(db.get_session_with_messages(999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn deleting_session_cascades_to_messages() {
        let db = test_db();
        let id = db.create_session("m", None).await.unwrap();
        db.add_message(id, "user", "hi", None).await.unwrap();

        db.delete_session(id).await.unwrap();

        assert!(db.get_messages_for_session(id).await.unwrap().is_empty());
    }

    #[test]
    fn migrates_old_messages_table_without_thinking() {
        let conn = Connection::open_in_memory().unwrap();
        // Simulate a database created before reasoning was stored
        conn.execute(
            "CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chat_messages (session_id, role, content) VALUES (1, 'user', 'hi')",
            [],
        )
        .unwrap();

        ChatDatabase::initialize_database(&conn).unwrap();

        let (content, thinking): (String, Option<String>) = conn
            .query_row(
                "SELECT content, thinking FROM chat_messages WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(content, "hi");
        assert_eq!(thinking, None);
    }

    #[tokio::test]
    async fn session_settings_roundtrip() {
        let db = test_db();
        let id = db.create_session("m", None).await.unwrap();
        assert!(db.get_sessions().await.unwrap()[0].settings.is_none());

        db.update_session_settings(id, r#"{"mode":"agent","code":true}"#)
            .await
            .unwrap();

        let session = db
            .get_session_with_messages(id)
            .await
            .unwrap()
            .unwrap()
            .session;
        assert_eq!(
            session.settings.as_deref(),
            Some(r#"{"mode":"agent","code":true}"#)
        );
    }
}
