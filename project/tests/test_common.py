"""Unit tests for common schemas, config, and constants."""

import unittest
from project.src.common.schemas import (
    AgentRole,
    AgentStatus,
    AgentExecutionStatus,
    TaskItem,
    TaskRequest,
    TaskResult,
    TaskStatus,
)
from project.src.common.constants import DEFAULT_WORKER_PORTS, ROLE_COMMAND_ALLOWLIST
from project.src.common.config import parse_simple_yaml


class TestCommon(unittest.TestCase):
    def test_task_request_serialization(self):
        req = TaskRequest(
            task_id="TASK-001",
            type="implementation",
            title="Implement CLI",
            description="Build CLI parser",
            repository="/workspace",
            branch="agent-a/task-001",
            timeout_seconds=120,
            context_files=["src/main.py"],
        )
        json_str = req.to_json()
        deserialized = TaskRequest.from_json(json_str)
        self.assertEqual(deserialized.task_id, "TASK-001")
        self.assertEqual(deserialized.type, "implementation")
        self.assertEqual(deserialized.context_files, ["src/main.py"])

    def test_task_result_serialization(self):
        res = TaskResult(
            task_id="TASK-001",
            agent_id="agent-a",
            status="SUCCESS",
            summary="All done",
            files_changed=["src/main.py"],
            commit="a1b2c3d",
        )
        json_str = res.to_json()
        deserialized = TaskResult.from_json(json_str)
        self.assertEqual(deserialized.status, "SUCCESS")
        self.assertEqual(deserialized.commit, "a1b2c3d")

    def test_agent_status(self):
        st = AgentStatus(
            agent_id="agent-a",
            role=AgentRole.DEVELOPER,
            status=AgentExecutionStatus.IDLE,
            host="127.0.0.1",
            port=8001,
        )
        json_str = st.to_json()
        deserialized = AgentStatus.from_json(json_str)
        self.assertEqual(deserialized.role, AgentRole.DEVELOPER)
        self.assertEqual(deserialized.status, AgentExecutionStatus.IDLE)

    def test_simple_yaml_parser(self):
        yaml_text = """
name: test-project
port: 8000
enabled: true
commands:
  - git
  - cargo
"""
        parsed = parse_simple_yaml(yaml_text)
        self.assertEqual(parsed.get("name"), "test-project")
        self.assertEqual(parsed.get("port"), 8000)
        self.assertEqual(parsed.get("enabled"), True)
        self.assertEqual(parsed.get("commands"), ["git", "cargo"])


if __name__ == "__main__":
    unittest.main()
