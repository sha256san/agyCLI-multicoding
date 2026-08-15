//! SQLite storage implementation for `mag`.

use chrono::{DateTime, Utc};
use mag_common::{AgentRole, TaskResult, TaskStatus};
use mag_task::Task;
use rusqlite::{params, Connection, Result as SqlResult};
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_task_crud() {
        let file = NamedTempFile::new().unwrap();
        let storage = Storage::new(file.path()).unwrap();

        let task = Task::new("T1", "Task 1", "Desc", "agent-a", AgentRole::Developer);
        storage.save_task(&task).unwrap();

        let loaded = storage.get_task("T1").unwrap().unwrap();
        assert_eq!(loaded.id, "T1");
        assert_eq!(loaded.title, "Task 1");

        let all = storage.list_tasks().unwrap();
        assert_eq!(all.len(), 1);
    }
}
