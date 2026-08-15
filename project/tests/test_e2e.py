"""End-to-End integration test for autonomous multi-agent orchestration."""

import os
import shutil
import tempfile
import threading
import time
import unittest

from project.src.common.schemas import AgentRole, TaskStatus
from project.src.manager.orchestrator import Orchestrator
from project.src.worker.server import run_worker_server


class TestMultiAgentE2E(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.test_ports = {
            "agent-a": 8091,
            "agent-b": 8092,
            "agent-c": 8093,
            "agent-d": 8094,
            "agent-e": 8095,
        }
        cls.worker_threads = []
        agents = [
            ("agent-a", AgentRole.DEVELOPER, 8091),
            ("agent-b", AgentRole.TESTER, 8092),
            ("agent-c", AgentRole.REVIEWER, 8093),
            ("agent-d", AgentRole.SECURITY, 8094),
            ("agent-e", AgentRole.RESEARCHER, 8095),
        ]
        for agent_id, role, port in agents:
            t = threading.Thread(
                target=run_worker_server,
                kwargs={"agent_id": agent_id, "role": role, "host": "127.0.0.1", "port": port},
                daemon=True,
            )
            t.start()
            cls.worker_threads.append(t)

        time.sleep(0.5)

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "e2e.sqlite")
        self.config_path = os.path.join(self.temp_dir, "project.yaml")

        # Write test config pointing to test ports
        with open(self.config_path, "w") as f:
            f.write(f"""
name: "e2e-project"
manager:
  max_retries: 3
agents:
  developer:
    id: "agent-a"
    port: {self.test_ports['agent-a']}
  tester:
    id: "agent-b"
    port: {self.test_ports['agent-b']}
  reviewer:
    id: "agent-c"
    port: {self.test_ports['agent-c']}
  security:
    id: "agent-d"
    port: {self.test_ports['agent-d']}
  researcher:
    id: "agent-e"
    port: {self.test_ports['agent-e']}
""")

    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_full_autonomous_orchestration_cycle(self):
        orch = Orchestrator(
            config_path=self.config_path,
            db_path=self.db_path,
            repo_path=self.temp_dir,
        )

        # 1. Verify health of all 5 workers
        health = orch.check_all_agents_health()
        for agent_id in ["agent-a", "agent-b", "agent-c", "agent-d", "agent-e"]:
            self.assertTrue(health.get(agent_id), f"Agent {agent_id} is not healthy")

        # 2. Decompose a natural language requirement
        user_prompt = "Build a lightweight key-value store module in Python"
        tasks = orch.decompose_requirement(user_prompt)
        self.assertEqual(len(tasks), 5)

        # 3. Run autonomous orchestration loop
        success = orch.run_orchestration_loop(max_iterations=20)
        self.assertTrue(success)

        # 4. Verify all tasks reached COMPLETED status
        all_tasks = orch.task_manager.list_tasks()
        for t in all_tasks:
            self.assertEqual(t.status, TaskStatus.COMPLETED, f"Task {t.task_id} status is {t.status}")
            self.assertIsNotNone(t.result)
            self.assertEqual(t.result.status, "SUCCESS")


if __name__ == "__main__":
    unittest.main()
