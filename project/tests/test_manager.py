"""Unit tests for Manager engine components."""

import os
import shutil
import tempfile
import unittest

from project.src.common.schemas import AgentRole, TaskItem, TaskResult, TaskStatus
from project.src.manager.db import DatabaseManager
from project.src.manager.diagnostics import EnvDoctor, JpCargoAnalyzer
from project.src.manager.evaluator import EvaluationVerdict, ResultEvaluator
from project.src.manager.git_manager import GitManager
from project.src.manager.task_manager import TaskManager


class TestManager(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "test.sqlite")
        self.db = DatabaseManager(db_path=self.db_path)
        self.tm = TaskManager(db=self.db)

    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_database_and_task_crud(self):
        t = self.tm.add_task(
            task_id="TASK-001",
            title="Design system",
            description="Design architecture",
            assigned_agent="agent-e",
            role=AgentRole.RESEARCHER,
        )
        self.assertEqual(t.status, TaskStatus.PENDING)

        fetched = self.tm.get_task("TASK-001")
        self.assertIsNotNone(fetched)
        self.assertEqual(fetched.title, "Design system")

        self.tm.update_task_status("TASK-001", TaskStatus.COMPLETED)
        updated = self.tm.get_task("TASK-001")
        self.assertEqual(updated.status, TaskStatus.COMPLETED)

    def test_task_dag_dependency_resolution(self):
        t1 = self.tm.add_task(task_id="T1", title="Task 1", description="Root task")
        t2 = self.tm.add_task(task_id="T2", title="Task 2", description="Child task", dependencies=["T1"])

        ready = self.tm.get_next_ready_tasks()
        ready_ids = [t.task_id for t in ready]
        self.assertIn("T1", ready_ids)
        self.assertNotIn("T2", ready_ids)

        # Complete T1
        self.tm.update_task_status("T1", TaskStatus.COMPLETED)
        ready_after = self.tm.get_next_ready_tasks()
        ready_ids_after = [t.task_id for t in ready_after]
        self.assertIn("T2", ready_ids_after)

    def test_evaluator_pass_and_retry(self):
        evaluator = ResultEvaluator(max_retries=3)
        task = TaskItem(
            task_id="T1",
            title="Implement feature",
            description="Feature",
            assigned_agent="agent-a",
            role=AgentRole.DEVELOPER,
            retry_count=0,
            max_retries=3,
        )

        success_result = TaskResult(
            task_id="T1",
            agent_id="agent-a",
            status="SUCCESS",
            summary="Implemented successfully",
        )
        ev_pass = evaluator.evaluate_task_result(task, success_result)
        self.assertEqual(ev_pass.verdict, EvaluationVerdict.PASS)

        fail_result = TaskResult(
            task_id="T1",
            agent_id="agent-a",
            status="FAILED",
            summary="Compilation error",
            errors=["SyntaxError"],
        )
        ev_retry = evaluator.evaluate_task_result(task, fail_result)
        self.assertEqual(ev_retry.verdict, EvaluationVerdict.RETRY)

        # Reached max retries
        task.retry_count = 3
        ev_fail = evaluator.evaluate_task_result(task, fail_result)
        self.assertEqual(ev_fail.verdict, EvaluationVerdict.FAIL)

    def test_git_manager(self):
        git = GitManager(repo_path=self.temp_dir)
        git.init_repo()
        self.assertTrue(git.is_git_repo())

    def test_diagnostics_envdoctor_and_jpcargo(self):
        diag = EnvDoctor.diagnose_system()
        self.assertIn("python_version", diag)
        self.assertIn("tools", diag)

        stderr_sample = "error[E0382]: use of moved value: `x`\n borrowed as mutable"
        rust_diag = JpCargoAnalyzer.analyze_rust_error(stderr_sample)
        self.assertTrue(len(rust_diag) > 0)
        self.assertIn("ミュータブル（可変）借用の衝突", rust_diag[0]["explanation_ja"])


if __name__ == "__main__":
    unittest.main()
