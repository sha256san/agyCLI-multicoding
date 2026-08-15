"""Common schemas and data models for Multi-Agent Development Orchestrator."""

from __future__ import annotations
from dataclasses import dataclass, field, asdict
from enum import Enum
import json
from typing import Any, Dict, List, Optional


class AgentRole(str, Enum):
    MANAGER = "manager"
    DEVELOPER = "developer"
    TESTER = "tester"
    REVIEWER = "reviewer"
    SECURITY = "security"
    RESEARCHER = "researcher"


class TaskStatus(str, Enum):
    PENDING = "PENDING"
    ASSIGNED = "ASSIGNED"
    RUNNING = "RUNNING"
    REVIEW = "REVIEW"
    TESTING = "TESTING"
    COMPLETED = "COMPLETED"
    FAILED = "FAILED"
    RETRY = "RETRY"
    FAILED_PERMANENTLY = "FAILED_PERMANENTLY"


class AgentExecutionStatus(str, Enum):
    IDLE = "IDLE"
    RUNNING = "RUNNING"
    ERROR = "ERROR"
    STOPPED = "STOPPED"


@dataclass
class TaskRequest:
    task_id: str
    type: str
    title: str
    description: str
    repository: str
    branch: str
    timeout_seconds: int = 300
    context_files: List[str] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> TaskRequest:
        return cls(
            task_id=data.get("task_id", ""),
            type=data.get("type", "general"),
            title=data.get("title", ""),
            description=data.get("description", ""),
            repository=data.get("repository", ""),
            branch=data.get("branch", "main"),
            timeout_seconds=data.get("timeout_seconds", 300),
            context_files=data.get("context_files", []),
            metadata=data.get("metadata", {}),
        )

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), ensure_ascii=False, indent=2)

    @classmethod
    def from_json(cls, json_str: str) -> TaskRequest:
        return cls.from_dict(json.loads(json_str))


@dataclass
class TaskResult:
    task_id: str
    agent_id: str
    status: str  # "SUCCESS" | "FAILED"
    summary: str
    files_changed: List[str] = field(default_factory=list)
    tests: List[Dict[str, Any]] = field(default_factory=list)
    commit: Optional[str] = None
    errors: List[str] = field(default_factory=list)
    execution_time_sec: float = 0.0
    output_details: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> TaskResult:
        return cls(
            task_id=data.get("task_id", ""),
            agent_id=data.get("agent_id", ""),
            status=data.get("status", "FAILED"),
            summary=data.get("summary", ""),
            files_changed=data.get("files_changed", []),
            tests=data.get("tests", []),
            commit=data.get("commit"),
            errors=data.get("errors", []),
            execution_time_sec=float(data.get("execution_time_sec", 0.0)),
            output_details=data.get("output_details", {}),
        )

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), ensure_ascii=False, indent=2)

    @classmethod
    def from_json(cls, json_str: str) -> TaskResult:
        return cls.from_dict(json.loads(json_str))


@dataclass
class AgentStatus:
    agent_id: str
    role: AgentRole
    status: AgentExecutionStatus
    current_task_id: Optional[str] = None
    started_at: Optional[str] = None
    progress_percent: int = 0
    host: str = "127.0.0.1"
    port: int = 8000

    def to_dict(self) -> Dict[str, Any]:
        d = asdict(self)
        d["role"] = self.role.value if isinstance(self.role, AgentRole) else str(self.role)
        d["status"] = self.status.value if isinstance(self.status, AgentExecutionStatus) else str(self.status)
        return d

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> AgentStatus:
        role_raw = data.get("role", "developer")
        status_raw = data.get("status", "IDLE")
        return cls(
            agent_id=data.get("agent_id", ""),
            role=AgentRole(role_raw) if role_raw in [r.value for r in AgentRole] else AgentRole.DEVELOPER,
            status=AgentExecutionStatus(status_raw) if status_raw in [s.value for s in AgentExecutionStatus] else AgentExecutionStatus.IDLE,
            current_task_id=data.get("current_task_id"),
            started_at=data.get("started_at"),
            progress_percent=int(data.get("progress_percent", 0)),
            host=data.get("host", "127.0.0.1"),
            port=int(data.get("port", 8000)),
        )

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), ensure_ascii=False, indent=2)

    @classmethod
    def from_json(cls, json_str: str) -> AgentStatus:
        return cls.from_dict(json.loads(json_str))


@dataclass
class TaskItem:
    task_id: str
    title: str
    description: str
    assigned_agent: str
    role: AgentRole
    priority: str = "medium"  # "low", "medium", "high"
    status: TaskStatus = TaskStatus.PENDING
    dependencies: List[str] = field(default_factory=list)
    retry_count: int = 0
    max_retries: int = 3
    result: Optional[TaskResult] = None
    created_at: str = ""
    updated_at: str = ""

    def to_dict(self) -> Dict[str, Any]:
        d = asdict(self)
        d["role"] = self.role.value if isinstance(self.role, AgentRole) else str(self.role)
        d["status"] = self.status.value if isinstance(self.status, TaskStatus) else str(self.status)
        return d

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> TaskItem:
        role_raw = data.get("role", "developer")
        status_raw = data.get("status", "PENDING")
        res_data = data.get("result")
        res_obj = TaskResult.from_dict(res_data) if res_data else None

        return cls(
            task_id=data.get("task_id", ""),
            title=data.get("title", ""),
            description=data.get("description", ""),
            assigned_agent=data.get("assigned_agent", "agent-a"),
            role=AgentRole(role_raw) if role_raw in [r.value for r in AgentRole] else AgentRole.DEVELOPER,
            priority=data.get("priority", "medium"),
            status=TaskStatus(status_raw) if status_raw in [s.value for s in TaskStatus] else TaskStatus.PENDING,
            dependencies=data.get("dependencies", []),
            retry_count=int(data.get("retry_count", 0)),
            max_retries=int(data.get("max_retries", 3)),
            result=res_obj,
            created_at=data.get("created_at", ""),
            updated_at=data.get("updated_at", ""),
        )
