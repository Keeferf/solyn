// src-tauri/src/data/chat/database.rs
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: i64,
    pub title: String,
    pub model_name: String,
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
            conn: Arc::new(Mutex::new(conn))
        })
    }
    
    fn initialize_database(conn: &Connection) -> Result<(), String> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT DEFAULT 'New Chat',
                model_name TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| e.to_string())?;
        
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
        ).map_err(|e| e.to_string())?;

        // Add thinking column to databases created before reasoning was stored
        let has_thinking = {
            let mut stmt = conn
                .prepare("PRAGMA table_info(chat_messages)")
                .map_err(|e| e.to_string())?;
            let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
            let mut found = false;
            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let name: String = row.get(1).map_err(|e| e.to_string())?;
                if name == "thinking" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_thinking {
            conn.execute("ALTER TABLE chat_messages ADD COLUMN thinking TEXT", [])
                .map_err(|e| e.to_string())?;
        }
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session ON chat_messages(session_id)",
            [],
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_updated ON chat_sessions(updated_at)",
            [],
        ).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    // Session operations
    pub async fn create_session(&self, model_name: &str, title: Option<&str>) -> Result<i64, String> {
        let title = title.unwrap_or("New Chat");
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO chat_sessions (title, model_name) VALUES (?1, ?2)",
            params![title, model_name],
        ).map_err(|e| e.to_string())?;
        
        Ok(conn.last_insert_rowid())
    }
    
    pub async fn get_sessions(&self) -> Result<Vec<ChatSession>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, title, model_name, created_at, updated_at 
             FROM chat_sessions 
             ORDER BY updated_at DESC"
        ).map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map([], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                model_name: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row.map_err(|e| e.to_string())?);
        }
        Ok(sessions)
    }
    
    pub async fn get_session_with_messages(&self, session_id: i64) -> Result<Option<ChatSessionWithMessages>, String> {
        // Get the session first
        let session = {
            let conn = self.conn.lock().await;
            let mut stmt = conn.prepare(
                "SELECT id, title, model_name, created_at, updated_at 
                 FROM chat_sessions 
                 WHERE id = ?1"
            ).map_err(|e| e.to_string())?;
            
            let mut rows = stmt.query(params![session_id]).map_err(|e| e.to_string())?;
            
            if let Some(row) = rows.next().map_err(|e| e.to_string())? {
                Some(ChatSession {
                    id: row.get(0).map_err(|e| e.to_string())?,
                    title: row.get(1).map_err(|e| e.to_string())?,
                    model_name: row.get(2).map_err(|e| e.to_string())?,
                    created_at: row.get(3).map_err(|e| e.to_string())?,
                    updated_at: row.get(4).map_err(|e| e.to_string())?,
                })
            } else {
                None
            }
        }; // All rusqlite types are dropped here before the await
        
        // Now get messages if we have a session
        if let Some(session) = session {
            let messages = self.get_messages_for_session(session_id).await?;
            Ok(Some(ChatSessionWithMessages {
                session,
                messages,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn update_session_title(&self, session_id: i64, title: &str) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE chat_sessions SET title = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![title, session_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
    
    pub async fn delete_session(&self, session_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM chat_sessions WHERE id = ?1",
            params![session_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
    
    // Message operations
    pub async fn add_message(&self, session_id: i64, role: &str, content: &str, thinking: Option<&str>) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        // Update session updated_at
        conn.execute(
            "UPDATE chat_sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![session_id],
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO chat_messages (session_id, role, content, thinking) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, role, content, thinking],
        ).map_err(|e| e.to_string())?;
        
        Ok(conn.last_insert_rowid())
    }
    
    pub async fn get_messages_for_session(&self, session_id: i64) -> Result<Vec<ChatMessage>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, thinking, created_at 
             FROM chat_messages 
             WHERE session_id = ?1 
             ORDER BY created_at ASC"
        ).map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                thinking: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row.map_err(|e| e.to_string())?);
        }
        Ok(messages)
    }
    
    pub async fn get_messages_for_session_since(&self, session_id: i64, since: i64) -> Result<Vec<ChatMessage>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, thinking, created_at 
             FROM chat_messages 
             WHERE session_id = ?1 AND id > ?2
             ORDER BY created_at ASC"
        ).map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map(params![session_id, since], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                thinking: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        
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
}