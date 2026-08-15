"""Main Multi-Agent Orchestrator engine."""

import os
import time
from typing import Any, Dict, List, Optional

from project.src.common.config import load_config_file
from project.src.common.constants import DEFAULT_MAX_RETRY, DEFAULT_WORKER_PORTS
from project.src.common.schemas import (
    AgentRole,
    AgentStatus,
    TaskItem,
    TaskRequest,
    TaskResult,
    TaskStatus,
)
from project.src.manager.db import DatabaseManager
from project.src.manager.evaluator import EvaluationVerdict, ResultEvaluator
from project.src.manager.git_manager import GitManager
from project.src.manager.task_manager import TaskManager
from project.src.manager.worker_client import WorkerClient


class Orchestrator:
    def __init__(
        self,
        config_path: str = "project/project.yaml",
        db_path: str = "logs/database.sqlite",
        repo_path: str = ".",
    ):
        self.config = load_config_file(config_path)
        self.db = DatabaseManager(db_path=db_path)
        self.task_manager = TaskManager(db=self.db)
        self.evaluator = ResultEvaluator(max_retries=self.config.get("manager", {}).get("max_retries", DEFAULT_MAX_RETRY))
        self.git_manager = GitManager(repo_path=repo_path)
        self.worker_client = WorkerClient()
        self.repo_path = os.path.abspath(repo_path)

        # Worker agent network mappings
        self.agents_config = self.config.get("agents", {})

    def get_agent_endpoint(self, agent_id_or_role: str) -> Dict[str, Any]:
        """Find host and port for an agent ID or role."""
        for role_name, info in self.agents_config.items():
            if isinstance(info, dict):
                if info.get("id") == agent_id_or_role or role_name == agent_id_or_role or info.get("role") == agent_id_or_role:
                    return {
                        "id": info.get("id", f"agent-{role_name}"),
                        "host": info.get("host", "127.0.0.1"),
                        "port": info.get("port", DEFAULT_WORKER_PORTS.get(info.get("id", ""), 8001)),
                        "role": info.get("role", role_name),
                    }

        default_port = DEFAULT_WORKER_PORTS.get(agent_id_or_role, 8001)
        return {
            "id": agent_id_or_role,
            "host": "127.0.0.1",
            "port": default_port,
            "role": "developer",
        }

    def check_all_agents_health(self) -> Dict[str, bool]:
        health_status = {}
        for role_name, info in self.agents_config.items():
            if isinstance(info, dict):
                agent_id = info.get("id", role_name)
                host = info.get("host", "127.0.0.1")
                port = info.get("port", 8001)
                is_healthy = self.worker_client.check_health(host, port)
                health_status[agent_id] = is_healthy
        return health_status

    def decompose_requirement(self, prompt: str, target_lang: str = "python") -> List[TaskItem]:
        """Decompose a high-level requirement into a standard task DAG:

        1. Research/Doc -> 2. Implementation -> 3. Testing
        -> 4. Review -> 5. Security
        """
        all_existing = self.task_manager.list_tasks()
        base_num = len(all_existing) + 1

        id1 = f"TASK-{base_num:03d}"
        id2 = f"TASK-{base_num + 1:03d}"
        id3 = f"TASK-{base_num + 2:03d}"
        id4 = f"TASK-{base_num + 3:03d}"
        id5 = f"TASK-{base_num + 4:03d}"

        tasks = []

        # 1. Research & Spec Task
        t1 = self.task_manager.add_task(
            task_id=id1,
            title=f"Spec & Architecture: {prompt[:50]}",
            description=f"Analyze requirements and produce architecture notes for: {prompt}",
            assigned_agent="agent-e",
            role=AgentRole.RESEARCHER,
            priority="high",
            dependencies=[],
        )
        tasks.append(t1)

        # 2. Developer Implementation Task
        t2 = self.task_manager.add_task(
            task_id=id2,
            title=f"Implementation: {prompt[:50]}",
            description=f"Implement source code for: {prompt}",
            assigned_agent="agent-a",
            role=AgentRole.DEVELOPER,
            priority="high",
            dependencies=[id1],
        )
        tasks.append(t2)

        # 3. Tester Verification Task
        t3 = self.task_manager.add_task(
            task_id=id3,
            title=f"Testing & Build: {prompt[:50]}",
            description=f"Run test suite and verify build for implementation of: {prompt}",
            assigned_agent="agent-b",
            role=AgentRole.TESTER,
            priority="high",
            dependencies=[id2],
        )
        tasks.append(t3)

        # 4. Reviewer Code Review Task
        t4 = self.task_manager.add_task(
            task_id=id4,
            title=f"Code Review: {prompt[:50]}",
            description=f"Perform static analysis and review code quality for: {prompt}",
            assigned_agent="agent-c",
            role=AgentRole.REVIEWER,
            priority="medium",
            dependencies=[id3],
        )
        tasks.append(t4)

        # 5. Security Scan Task
        t5 = self.task_manager.add_task(
            task_id=id5,
            title=f"Security Audit: {prompt[:50]}",
            description=f"Audit vulnerabilities, secret leaks, and security posture for: {prompt}",
            assigned_agent="agent-d",
            role=AgentRole.SECURITY,
            priority="medium",
            dependencies=[id4],
        )
        tasks.append(t5)

        return tasks

    def execute_single_task(self, task: TaskItem, metadata: Optional[Dict[str, Any]] = None) -> TaskResult:
        endpoint = self.get_agent_endpoint(task.assigned_agent)
        host = endpoint["host"]
        port = endpoint["port"]

        self.task_manager.update_task_status(task.task_id, TaskStatus.RUNNING)

        task_req = TaskRequest(
            task_id=task.task_id,
            type=task.role.value if hasattr(task.role, "value") else str(task.role),
            title=task.title,
            description=task.description,
            repository=self.repo_path,
            branch=f"{task.assigned_agent}/{task.task_id.lower()}",
            timeout_seconds=300,
            metadata=metadata or {},
        )

        # Send task
        send_res = self.worker_client.send_task(host, port, task_req)
        if "error" in send_res:
            # Worker not reachable or error
            err_result = TaskResult(
                task_id=task.task_id,
                agent_id=task.assigned_agent,
                status="FAILED",
                summary=f"Failed to communicate with worker {task.assigned_agent} on {host}:{port}: {send_res.get('error')}",
                errors=[str(send_res.get("error"))],
            )
            self.task_manager.update_task_status(task.task_id, TaskStatus.FAILED, result=err_result, increment_retry=True)
            return err_result

        # Poll status until IDLE or completed
        max_poll_time = 300
        poll_interval = 0.5
        elapsed = 0.0

        while elapsed < max_poll_time:
            time.sleep(poll_interval)
            elapsed += poll_interval

            status = self.worker_client.get_status(host, port)
            if status and status.current_task_id != task.task_id:
                # Task finished
                result = self.worker_client.get_result(host, port)
                if result and result.task_id == task.task_id:
                    # Evaluate result
                    eval_res = self.evaluator.evaluate_task_result(task, result)
                    if eval_res.verdict == EvaluationVerdict.PASS:
                        self.task_manager.update_task_status(task.task_id, TaskStatus.COMPLETED, result=result)
                    elif eval_res.verdict == EvaluationVerdict.RETRY:
                        self.task_manager.update_task_status(task.task_id, TaskStatus.RETRY, result=result, increment_retry=True)
                    else:
                        self.task_manager.update_task_status(task.task_id, TaskStatus.FAILED_PERMANENTLY, result=result, increment_retry=True)
                    return result

        timeout_result = TaskResult(
            task_id=task.task_id,
            agent_id=task.assigned_agent,
            status="FAILED",
            summary=f"Task {task.task_id} timed out while polling worker {task.assigned_agent}",
            errors=["Timeout"],
        )
        self.task_manager.update_task_status(task.task_id, TaskStatus.FAILED, result=timeout_result, increment_retry=True)
        return timeout_result

    def run_orchestration_loop(self, max_iterations: int = 50) -> bool:
        """Run tasks in dependency order until all are completed or permanently failed."""
        iterations = 0

        while iterations < max_iterations:
            iterations += 1
            ready_tasks = self.task_manager.get_next_ready_tasks()
            if not ready_tasks:
                break

            for task in ready_tasks:
                print(f"[*] Dispatching task [{task.task_id}] '{task.title}' to [{task.assigned_agent}]...")
                result = self.execute_single_task(task)
                print(f"    -> Result: {result.status} | {result.summary}")

        all_done = self.task_manager.is_all_completed()
        return all_done
