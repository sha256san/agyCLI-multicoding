"""Task Manager and DAG scheduler for Multi-Agent Orchestrator."""

from datetime import datetime, timezone
from typing import Dict, List, Optional

from project.src.common.schemas import AgentRole, TaskItem, TaskResult, TaskStatus
from project.src.manager.db import DatabaseManager


class TaskManager:
    def __init__(self, db: Optional[DatabaseManager] = None):
        self.db = db or DatabaseManager()

    def add_task(
        self,
        task_id: str,
        title: str,
        description: str,
        assigned_agent: str = "agent-a",
        role: AgentRole = AgentRole.DEVELOPER,
        priority: str = "medium",
        dependencies: Optional[List[str]] = None,
        max_retries: int = 3,
    ) -> TaskItem:
        task = TaskItem(
            task_id=task_id,
            title=title,
            description=description,
            assigned_agent=assigned_agent,
            role=role,
            priority=priority,
            status=TaskStatus.PENDING,
            dependencies=dependencies or [],
            retry_count=0,
            max_retries=max_retries,
            created_at=datetime.now(timezone.utc).isoformat(),
            updated_at=datetime.now(timezone.utc).isoformat(),
        )
        self.db.save_task(task)
        self.db.add_log("INFO", f"Registered task '{task_id}': {title}", task_id=task_id)
        return task

    def get_task(self, task_id: str) -> Optional[TaskItem]:
        return self.db.get_task(task_id)

    def list_tasks(self, status: Optional[TaskStatus] = None) -> List[TaskItem]:
        return self.db.list_tasks(status=status)

    def update_task_status(
        self,
        task_id: str,
        status: TaskStatus,
        result: Optional[TaskResult] = None,
        increment_retry: bool = False,
    ) -> Optional[TaskItem]:
        task = self.db.get_task(task_id)
        if not task:
            return None

        task.status = status
        if result:
            task.result = result
        if increment_retry:
            task.retry_count += 1
        task.updated_at = datetime.now(timezone.utc).isoformat()

        self.db.save_task(task)
        self.db.add_log(
            "INFO",
            f"Task '{task_id}' transitioned to status '{status.value}' (retries: {task.retry_count})",
            task_id=task_id,
            agent_id=task.assigned_agent,
        )
        return task

    def get_next_ready_tasks(self) -> List[TaskItem]:
        """Find pending or retry tasks whose dependencies are all COMPLETED."""
        all_tasks = self.db.list_tasks()
        completed_ids = {t.task_id for t in all_tasks if t.status == TaskStatus.COMPLETED}

        ready_tasks = []
        for t in all_tasks:
            if t.status in (TaskStatus.PENDING, TaskStatus.RETRY):
                deps_met = all(dep in completed_ids for dep in t.dependencies)
                if deps_met:
                    ready_tasks.append(t)

        return ready_tasks

    def is_all_completed(self) -> bool:
        tasks = self.db.list_tasks()
        if not tasks:
            return True
        return all(t.status == TaskStatus.COMPLETED for t in tasks)
