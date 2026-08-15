"""Unit and integration tests for Worker Agent components."""

import os
import shutil
import tempfile
import threading
import time
import unittest

from project.src.common.schemas import AgentRole, TaskRequest
from project.src.worker.executor import CommandExecutor
from project.src.worker.agent_logic import create_agent_handler
from project.src.worker.server import run_worker_server
from project.src.manager.worker_client import WorkerClient


class TestWorker(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_command_executor_safety(self):
        exec_dev = CommandExecutor(role="developer", allowlist=["python3", "ls", "echo"])
        is_safe, _ = exec_dev.is_command_safe("python3 --version")
        self.assertTrue(is_safe)

        # Dangerous keyword rejection
        is_safe, msg = exec_dev.is_command_safe("rm -rf /")
        self.assertFalse(is_safe)
        self.assertIn("dangerous", msg.lower())

        # Unallowed command rejection
        is_safe, msg = exec_dev.is_command_safe("nc -l 9999")
        self.assertFalse(is_safe)
        self.assertIn("not in the allowlist", msg)

    def test_developer_handler_file_creation(self):
        handler = create_agent_handler("agent-a", AgentRole.DEVELOPER)
        task = TaskRequest(
            task_id="TASK-T1",
            type="developer",
            title="Create hello module",
            description="Create hello.py",
            repository=self.temp_dir,
            branch="test-branch",
            metadata={
                "files": {
                    "hello.py": "def hello(): return 'world'\n"
                }
            }
        )
        result = handler.execute_task(task)
        self.assertEqual(result.status, "SUCCESS")
        self.assertIn("hello.py", result.files_changed)
        self.assertTrue(os.path.exists(os.path.join(self.temp_dir, "hello.py")))

    def test_security_handler_secret_leak_detection(self):
        # Create a file with hardcoded secret
        secret_file = os.path.join(self.temp_dir, "config.py")
        with open(secret_file, "w") as f:
            f.write("API_KEY = 'secret_token_12345678'\n")

        sec_handler = create_agent_handler("agent-d", AgentRole.SECURITY)
        task = TaskRequest(
            task_id="TASK-S1",
            type="security",
            title="Scan secrets",
            description="Scan config.py",
            repository=self.temp_dir,
            branch="main",
        )
        result = sec_handler.execute_task(task)
        self.assertEqual(result.status, "FAILED")
        self.assertFalse(result.output_details.get("secure"))
        self.assertTrue(len(result.output_details.get("findings")) > 0)

    def test_worker_rest_server_lifecycle(self):
        test_port = 8123
        server_thread = threading.Thread(
            target=run_worker_server,
            kwargs={"agent_id": "test-agent", "role": AgentRole.DEVELOPER, "host": "127.0.0.1", "port": test_port},
            daemon=True,
        )
        server_thread.start()
        time.sleep(0.3)

        client = WorkerClient()
        is_healthy = client.check_health("127.0.0.1", test_port)
        self.assertTrue(is_healthy)

        status = client.get_status("127.0.0.1", test_port)
        self.assertIsNotNone(status)
        self.assertEqual(status.agent_id, "test-agent")

        # Submit task
        task = TaskRequest(
            task_id="TASK-HTTP-1",
            type="developer",
            title="HTTP test task",
            description="Test task via REST",
            repository=self.temp_dir,
            branch="main",
        )
        send_res = client.send_task("127.0.0.1", test_port, task)
        self.assertIn("message", send_res)

        # Wait for task completion
        time.sleep(0.5)
        res = client.get_result("127.0.0.1", test_port)
        self.assertIsNotNone(res)
        self.assertEqual(res.task_id, "TASK-HTTP-1")
        self.assertEqual(res.status, "SUCCESS")


if __name__ == "__main__":
    unittest.main()
