//! SQLite storage implementation for `mag`.

use chrono::{DateTime, Utc};
use mag_common::{AgentRole, TaskResult, TaskStatus};
use mag_task::Task;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("SQLite database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Structured event record for the event log and replay stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: i64,
    pub task_id: String,
    pub agent_id: String,
    pub event_type: String,
    pub payload_json: String,
    pub created_at: DateTime<Utc>,
}

/// Structured session record for terminal attach/detach tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub task_id: String,
    pub status: String, // "ATTACHED" | "DETACHED" | "CLOSED"
    pub created_at: DateTime<Utc>,
    pub last_attached_at: Option<DateTime<Utc>>,
    pub last_detached_at: Option<DateTime<Utc>>,
}

/// Structured agent runtime record in database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub role: String,
    pub status: String,
    pub container_id: Option<String>,
    pub current_task: Option<String>,
    pub last_heartbeat: DateTime<Utc>,
}

/// SQLite persistence manager.
pub struct Storage {
    db_path: String,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_string_lossy().to_string();
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let storage = Self { db_path };
        storage.init_db()?;
        Ok(storage)
    }

    fn connect(&self) -> SqlResult<Connection> {
        Connection::open(&self.db_path)
    }

    fn init_db(&self) -> Result<(), StorageError> {
        let conn = self.connect()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                assigned_agent TEXT,
                role TEXT,
                priority TEXT,
                status TEXT,
                dependencies TEXT,
                retry_count INTEGER,
                max_retries INTEGER,
                result_json TEXT,
                created_at TEXT,
                updated_at TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_attached_at TEXT,
                last_detached_at TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                role TEXT NOT NULL,
                status TEXT NOT NULL,
                container_id TEXT,
                current_task TEXT,
                last_heartbeat TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                level TEXT,
                component TEXT,
                agent_id TEXT,
                task_id TEXT,
                message TEXT
            )",
            [],
        )?;

        Ok(())
    }

    // -------------------------------------------------------------
    // Task Operations
    // -------------------------------------------------------------

    pub fn save_task(&self, task: &Task) -> Result<(), StorageError> {
        let conn = self.connect()?;
        let deps_json = serde_json::to_string(&task.dependencies)?;
        let result_json = task
            .result
            .as_ref()
            .map(|r| serde_json::to_string(r))
            .transpose()?;

        conn.execute(
            "INSERT INTO tasks (
                id, title, description, assigned_agent, role,
                priority, status, dependencies, retry_count, max_retries,
                result_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                title=excluded.title,
                description=excluded.description,
                assigned_agent=excluded.assigned_agent,
                role=excluded.role,
                priority=excluded.priority,
                status=excluded.status,
                dependencies=excluded.dependencies,
                retry_count=excluded.retry_count,
                max_retries=excluded.max_retries,
                result_json=excluded.result_json,
                updated_at=excluded.updated_at",
            params![
                task.id,
                task.title,
                task.description,
                task.assigned_agent,
                task.role.to_string(),
                task.priority,
                task.status.to_string(),
                deps_json,
                task.retry_count,
                task.max_retries,
                result_json,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Result<Option<Task>, StorageError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT * FROM tasks WHERE id = ?1")?;
        let mut rows = stmt.query(params![task_id])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get("id")?;
            let title: String = row.get("title")?;
            let description: String = row.get("description")?;
            let assigned_agent: String = row.get("assigned_agent")?;
            let role_str: String = row.get("role")?;
            let priority: String = row.get("priority")?;
            let status_str: String = row.get("status")?;
            let deps_json: String = row.get("dependencies")?;
            let retry_count: u32 = row.get("retry_count")?;
            let max_retries: u32 = row.get("max_retries")?;
            let result_json: Option<String> = row.get("result_json")?;
            let created_at_str: String = row.get("created_at")?;
            let updated_at_str: String = row.get("updated_at")?;

            let role: AgentRole = serde_json::from_value(serde_json::Value::String(role_str)).unwrap_or(AgentRole::Developer);
            let status: TaskStatus = serde_json::from_value(serde_json::Value::String(status_str)).unwrap_or(TaskStatus::Pending);
            let dependencies: Vec<String> = serde_json::from_str(&deps_json).unwrap_or_default();
            let result: Option<TaskResult> = result_json.and_then(|s| serde_json::from_str(&s).ok());
            let created_at = DateTime::parse_from_rfc3339(&created_at_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());

            Ok(Some(Task {
                id,
                title,
                description,
                assigned_agent,
                role,
                priority,
                status,
                dependencies,
                retry_count,
                max_retries,
                result,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>, StorageError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT * FROM tasks ORDER BY created_at ASC")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get("id")?;
            let title: String = row.get("title")?;
            let description: String = row.get("description")?;
            let assigned_agent: String = row.get("assigned_agent")?;
            let role_str: String = row.get("role")?;
            let priority: String = row.get("priority")?;
            let status_str: String = row.get("status")?;
            let deps_json: String = row.get("dependencies")?;
            let retry_count: u32 = row.get("retry_count")?;
            let max_retries: u32 = row.get("max_retries")?;
            let result_json: Option<String> = row.get("result_json")?;
            let created_at_str: String = row.get("created_at")?;
            let updated_at_str: String = row.get("updated_at")?;

            let role: AgentRole = serde_json::from_value(serde_json::Value::String(role_str)).unwrap_or(AgentRole::Developer);
            let status: TaskStatus = serde_json::from_value(serde_json::Value::String(status_str)).unwrap_or(TaskStatus::Pending);
            let dependencies: Vec<String> = serde_json::from_str(&deps_json).unwrap_or_default();
            let result: Option<TaskResult> = result_json.and_then(|s| serde_json::from_str(&s).ok());
            let created_at = DateTime::parse_from_rfc3339(&created_at_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());

            Ok(Task {
                id,
                title,
                description,
                assigned_agent,
                role,
                priority,
                status,
                dependencies,
                retry_count,
                max_retries,
                result,
                created_at,
                updated_at,
            })
        })?;

        let mut tasks = Vec::new();
        for task in rows {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    // -------------------------------------------------------------
    // Event Store Operations
    // -------------------------------------------------------------

    pub fn record_event<T: Serialize>(
        &self,
        task_id: &str,
        agent_id: &str,
        event_type: &str,
        payload: &T,
    ) -> Result<i64, StorageError> {
        let conn = self.connect()?;
        let payload_json = serde_json::to_string(payload)?;
        let created_at = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO events (task_id, agent_id, event_type, payload_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![task_id, agent_id, event_type, payload_json, created_at],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_events(&self, task_id: &str) -> Result<Vec<EventRecord>, StorageError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, task_id, agent_id, event_type, payload_json, created_at
             FROM events WHERE task_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![task_id], |row| {
            let id: i64 = row.get("id")?;
            let task_id: String = row.get("task_id")?;
            let agent_id: String = row.get("agent_id")?;
            let event_type: String = row.get("event_type")?;
            let payload_json: String = row.get("payload_json")?;
            let created_at_str: String = row.get("created_at")?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(EventRecord {
                id,
                task_id,
                agent_id,
                event_type,
                payload_json,
                created_at,
            })
        })?;

        let mut events = Vec::new();
        for event in rows {
            events.push(event?);
        }
        Ok(events)
    }

    // -------------------------------------------------------------
    // Session Operations (Attach / Detach)
    // -------------------------------------------------------------

    pub fn create_session(&self, task_id: &str) -> Result<SessionRecord, StorageError> {
        let conn = self.connect()?;
        let session_id = format!("sess_{}", Utc::now().timestamp_millis());
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        conn.execute(
            "INSERT INTO sessions (id, task_id, status, created_at, last_attached_at, last_detached_at)
             VALUES (?1, ?2, 'ATTACHED', ?3, ?3, NULL)",
            params![session_id, task_id, now_str],
        )?;

        Ok(SessionRecord {
            id: session_id,
            task_id: task_id.to_string(),
            status: "ATTACHED".to_string(),
            created_at: now,
            last_attached_at: Some(now),
            last_detached_at: None,
        })
    }

    pub fn attach_session(&self, task_id: &str) -> Result<SessionRecord, StorageError> {
        let conn = self.connect()?;
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let mut stmt = conn.prepare("SELECT * FROM sessions WHERE task_id = ?1 ORDER BY created_at DESC LIMIT 1")?;
        let mut rows = stmt.query(params![task_id])?;

        if let Some(row) = rows.next()? {
            let session_id: String = row.get("id")?;
            conn.execute(
                "UPDATE sessions SET status = 'ATTACHED', last_attached_at = ?1 WHERE id = ?2",
                params![now_str, session_id],
            )?;

            let created_at_str: String = row.get("created_at")?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(SessionRecord {
                id: session_id,
                task_id: task_id.to_string(),
                status: "ATTACHED".to_string(),
                created_at,
                last_attached_at: Some(now),
                last_detached_at: None,
            })
        } else {
            self.create_session(task_id)
        }
    }

    pub fn detach_session(&self, task_id: &str) -> Result<(), StorageError> {
        let conn = self.connect()?;
        let now_str = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET status = 'DETACHED', last_detached_at = ?1 WHERE task_id = ?2 AND status = 'ATTACHED'",
            params![now_str, task_id],
        )?;
        Ok(())
    }

    // -------------------------------------------------------------
    // Agent Operations & Heartbeat
    // -------------------------------------------------------------

    pub fn update_agent_heartbeat(
        &self,
        agent_id: &str,
        role: &str,
        status: &str,
        container_id: Option<&str>,
        current_task: Option<&str>,
    ) -> Result<(), StorageError> {
        let conn = self.connect()?;
        let now_str = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO agents (id, role, status, container_id, current_task, last_heartbeat)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                role=excluded.role,
                status=excluded.status,
                container_id=excluded.container_id,
                current_task=excluded.current_task,
                last_heartbeat=excluded.last_heartbeat",
            params![agent_id, role, status, container_id, current_task, now_str],
        )?;

        Ok(())
    }

    pub fn list_agents(&self) -> Result<Vec<AgentRecord>, StorageError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT * FROM agents ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get("id")?;
            let role: String = row.get("role")?;
            let status: String = row.get("status")?;
            let container_id: Option<String> = row.get("container_id")?;
            let current_task: Option<String> = row.get("current_task")?;
            let heartbeat_str: String = row.get("last_heartbeat")?;
            let last_heartbeat = DateTime::parse_from_rfc3339(&heartbeat_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(AgentRecord {
                id,
                role,
                status,
                container_id,
                current_task,
                last_heartbeat,
            })
        })?;

        let mut agents = Vec::new();
        for agent in rows {
            agents.push(agent?);
        }
        Ok(agents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_task_and_events() {
        let file = NamedTempFile::new().unwrap();
        let storage = Storage::new(file.path()).unwrap();

        let task = Task::new("T1", "Task 1", "Desc", "agent-a", AgentRole::Developer);
        storage.save_task(&task).unwrap();

        let loaded = storage.get_task("T1").unwrap().unwrap();
        assert_eq!(loaded.id, "T1");
        assert_eq!(loaded.title, "Task 1");

        // Event store
        storage.record_event("T1", "agent-a", "AGENT_STARTED", &"Starting work").unwrap();
        storage.record_event("T1", "agent-a", "CODE_CHANGED", &vec!["src/main.rs"]).unwrap();
        let events = storage.list_events("T1").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "AGENT_STARTED");

        // Session
        let sess = storage.create_session("T1").unwrap();
        assert_eq!(sess.status, "ATTACHED");
        storage.detach_session("T1").unwrap();

        // Heartbeat
        storage.update_agent_heartbeat("agent-a", "developer", "IDLE", Some("cnt-123"), None).unwrap();
        let agents = storage.list_agents().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "agent-a");
    }
}
