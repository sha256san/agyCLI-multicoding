"""Database persistence layer using SQLite for Multi-Agent Orchestrator."""

from contextlib import contextmanager
from datetime import datetime, timezone
import json
import os
import sqlite3
from typing import Any, Dict, Generator, List, Optional

from project.src.common.schemas import TaskItem, TaskResult, TaskStatus


class DatabaseManager:
    def __init__(self, db_path: str = "logs/database.sqlite"):
        self.db_path = db_path
        os.makedirs(os.path.dirname(db_path), exist_ok=True)
        self._init_db()

    @contextmanager
    def _connection(self) -> Generator[sqlite3.Connection, None, None]:
        conn = sqlite3.connect(self.db_path, timeout=10.0)
        conn.row_factory = sqlite3.Row
        try:
            yield conn
        finally:
            conn.close()

    def _init_db(self):
        with self._connection() as conn:
            cursor = conn.cursor()
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS tasks (
                    task_id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    description TEXT,
                    assigned_agent TEXT,
                    role TEXT,
                    priority TEXT,
                    status TEXT,
                    dependencies TEXT,
                    retry_count INTEGER DEFAULT 0,
                    max_retries INTEGER DEFAULT 3,
                    result_json TEXT,
                    created_at TEXT,
                    updated_at TEXT
                )
            """)

            cursor.execute("""
                CREATE TABLE IF NOT EXISTS agents (
                    agent_id TEXT PRIMARY KEY,
                    role TEXT NOT NULL,
                    host TEXT,
                    port INTEGER,
                    status TEXT,
                    current_task_id TEXT,
                    last_seen TEXT
                )
            """)

            cursor.execute("""
                CREATE TABLE IF NOT EXISTS logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL,
                    agent_id TEXT,
                    task_id TEXT,
                    level TEXT,
                    message TEXT
                )
            """)
            conn.commit()

    def save_task(self, task: TaskItem):
        now = datetime.now(timezone.utc).isoformat()
        if not task.created_at:
            task.created_at = now
        task.updated_at = now

        result_str = task.result.to_json() if task.result else None
        deps_str = json.dumps(task.dependencies)

        with self._connection() as conn:
            cursor = conn.cursor()
            cursor.execute("""
                INSERT INTO tasks (
                    task_id, title, description, assigned_agent, role,
                    priority, status, dependencies, retry_count, max_retries,
                    result_json, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(task_id) DO UPDATE SET
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
                    updated_at=excluded.updated_at
            """, (
                task.task_id, task.title, task.description, task.assigned_agent,
                task.role.value if hasattr(task.role, "value") else str(task.role),
                task.priority,
                task.status.value if hasattr(task.status, "value") else str(task.status),
                deps_str, task.retry_count, task.max_retries,
                result_str, task.created_at, task.updated_at
            ))
            conn.commit()

    def get_task(self, task_id: str) -> Optional[TaskItem]:
        with self._connection() as conn:
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM tasks WHERE task_id = ?", (task_id,))
            row = cursor.fetchone()
            if not row:
                return None

            data = dict(row)
            if data.get("dependencies"):
                try:
                    data["dependencies"] = json.loads(data["dependencies"])
                except Exception:
                    data["dependencies"] = []
            if data.get("result_json"):
                try:
                    data["result"] = json.loads(data["result_json"])
                except Exception:
                    data["result"] = None

            return TaskItem.from_dict(data)

    def list_tasks(self, status: Optional[TaskStatus] = None) -> List[TaskItem]:
        with self._connection() as conn:
            cursor = conn.cursor()
            if status:
                cursor.execute("SELECT * FROM tasks WHERE status = ? ORDER BY created_at ASC", (status.value,))
            else:
                cursor.execute("SELECT * FROM tasks ORDER BY created_at ASC")

            rows = cursor.fetchall()
            tasks = []
            for row in rows:
                data = dict(row)
                if data.get("dependencies"):
                    try:
                        data["dependencies"] = json.loads(data["dependencies"])
                    except Exception:
                        data["dependencies"] = []
                if data.get("result_json"):
                    try:
                        data["result"] = json.loads(data["result_json"])
                    except Exception:
                        data["result"] = None
                tasks.append(TaskItem.from_dict(data))
            return tasks

    def add_log(self, level: str, message: str, agent_id: Optional[str] = None, task_id: Optional[str] = None):
        now = datetime.now(timezone.utc).isoformat()
        with self._connection() as conn:
            cursor = conn.cursor()
            cursor.execute("""
                INSERT INTO logs (timestamp, agent_id, task_id, level, message)
                VALUES (?, ?, ?, ?, ?)
            """, (now, agent_id, task_id, level, message))
            conn.commit()

    def get_logs(self, agent_id: Optional[str] = None, task_id: Optional[str] = None, limit: int = 100) -> List[Dict[str, Any]]:
        with self._connection() as conn:
            cursor = conn.cursor()
            query = "SELECT * FROM logs WHERE 1=1"
            params: List[Any] = []
            if agent_id:
                query += " AND agent_id = ?"
                params.append(agent_id)
            if task_id:
                query += " AND task_id = ?"
                params.append(task_id)
            query += " ORDER BY id DESC LIMIT ?"
            params.append(limit)

            cursor.execute(query, tuple(params))
            return [dict(row) for row in cursor.fetchall()]
