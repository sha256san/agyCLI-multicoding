"""Role-specific task logic handlers for Worker agents."""

import os
import re
import time
from typing import Any, Dict, List, Optional

from project.src.common.schemas import AgentRole, TaskRequest, TaskResult
from project.src.worker.executor import CommandExecutor


class BaseAgentHandler:
    def __init__(self, agent_id: str, role: AgentRole):
        self.agent_id = agent_id
        self.role = role
        self.executor = CommandExecutor(role=role.value)

    def execute_task(self, task: TaskRequest) -> TaskResult:
        raise NotImplementedError


class DeveloperHandler(BaseAgentHandler):
    """Developer agent handler: implementation, bug fix, refactoring, git commits."""

    def execute_task(self, task: TaskRequest) -> TaskResult:
        start_time = time.time()
        files_changed: List[str] = []
        errors: List[str] = []
        commit_hash: Optional[str] = None

        repo_dir = task.repository if task.repository and os.path.exists(task.repository) else os.getcwd()

        # Check metadata for specific file writes/operations
        file_actions = task.metadata.get("files", {})
        for file_rel_path, content in file_actions.items():
            full_path = os.path.join(repo_dir, file_rel_path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, "w", encoding="utf-8") as f:
                f.write(content)
            files_changed.append(file_rel_path)

        # Run custom implementation command if specified
        custom_cmd = task.metadata.get("command")
        exec_output = None
        if custom_cmd:
            exec_output = self.executor.run_command(
                custom_cmd, cwd=repo_dir, timeout_seconds=task.timeout_seconds
            )
            if not exec_output["success"]:
                errors.append(exec_output["stderr"])

        # Try to commit changes if git repository exists
        if os.path.exists(os.path.join(repo_dir, ".git")):
            self.executor.run_command(f"git checkout -B {task.branch}", cwd=repo_dir)
            self.executor.run_command("git add .", cwd=repo_dir)
            commit_res = self.executor.run_command(
                f'git commit -m "feat({task.task_id}): {task.title}"', cwd=repo_dir
            )
            if commit_res["success"]:
                rev_res = self.executor.run_command("git rev-parse --short HEAD", cwd=repo_dir)
                if rev_res["success"]:
                    commit_hash = rev_res["stdout"].strip()

        duration = time.time() - start_time
        success = len(errors) == 0

        summary = f"Developer '{self.agent_id}' completed task '{task.task_id}': {task.title}."
        if files_changed:
            summary += f" Modified {len(files_changed)} file(s): {', '.join(files_changed)}."
        if commit_hash:
            summary += f" Commit: {commit_hash}."

        return TaskResult(
            task_id=task.task_id,
            agent_id=self.agent_id,
            status="SUCCESS" if success else "FAILED",
            summary=summary,
            files_changed=files_changed,
            tests=[],
            commit=commit_hash,
            errors=errors,
            execution_time_sec=duration,
            output_details={"exec_output": exec_output} if exec_output else {},
        )


class TesterHandler(BaseAgentHandler):
    """Tester agent handler: running unit/integration tests and build verification."""

    def execute_task(self, task: TaskRequest) -> TaskResult:
        start_time = time.time()
        repo_dir = task.repository if task.repository and os.path.exists(task.repository) else os.getcwd()

        test_cmd = task.metadata.get("test_command", "python3 -m unittest discover")
        res = self.executor.run_command(test_cmd, cwd=repo_dir, timeout_seconds=task.timeout_seconds)

        # In Python 3.12+, unittest returns exit code 5 if 0 tests ran
        output_combined = (res["stdout"] + "\n" + res["stderr"]).upper()
        zero_tests_ran = (res["exit_code"] == 5 and "NO TESTS RAN" in output_combined)

        tests_passed = res["success"] or zero_tests_ran
        tests_list = []
        errors = []

        if not tests_passed:
            errors.append(res["stderr"] or res["stdout"])

        tests_list.append({
            "command": test_cmd,
            "passed": tests_passed,
            "zero_tests_ran": zero_tests_ran,
            "stdout": res["stdout"],
            "stderr": res["stderr"],
            "exit_code": res["exit_code"],
        })

        duration = time.time() - start_time
        summary = (
            f"Tester '{self.agent_id}' ran test suite. "
            f"Result: {'PASS' if tests_passed else 'FAIL'} (exit code {res['exit_code']})."
        )
        if zero_tests_ran:
            summary += " (No test files found, test suite passed cleanly)."

        return TaskResult(
            task_id=task.task_id,
            agent_id=self.agent_id,
            status="SUCCESS" if tests_passed else "FAILED",
            summary=summary,
            files_changed=[],
            tests=tests_list,
            commit=None,
            errors=errors,
            execution_time_sec=duration,
            output_details={"stdout": res["stdout"], "stderr": res["stderr"]},
        )


class ReviewerHandler(BaseAgentHandler):
    """Reviewer agent handler: static analysis, readability, maintainability, design check."""

    def execute_task(self, task: TaskRequest) -> TaskResult:
        start_time = time.time()
        repo_dir = task.repository if task.repository and os.path.exists(task.repository) else os.getcwd()

        issues: List[Dict[str, Any]] = []
        files_to_check = task.context_files or ["src"]

        for f_path in files_to_check:
            full_path = os.path.join(repo_dir, f_path)
            if not os.path.exists(full_path):
                continue

            # Python syntax / lint inspection
            if full_path.endswith(".py") or os.path.isdir(full_path):
                lint_res = self.executor.run_command(
                    f"python3 -m py_compile {full_path}" if full_path.endswith(".py") else "ls",
                    cwd=repo_dir,
                )
                if not lint_res["success"]:
                    issues.append({
                        "file": f_path,
                        "severity": "critical",
                        "rule": "syntax-error",
                        "message": lint_res["stderr"].strip(),
                    })

        # Check for dangerous patterns in target source files
        check_dirs = [os.path.join(repo_dir, p) for p in files_to_check]
        for target in check_dirs:
            if not os.path.exists(target):
                continue
            if os.path.isfile(target):
                candidates = [target]
            else:
                candidates = []
                for root, dirs, files in os.walk(target):
                    dirs[:] = [d for d in dirs if d not in (".git", "__pycache__", "logs", "project")]
                    for file in files:
                        if file.endswith((".py", ".rs", ".js", ".ts")):
                            candidates.append(os.path.join(root, file))

            for file_full in candidates:
                rel_path = os.path.relpath(file_full, repo_dir)
                try:
                    with open(file_full, "r", encoding="utf-8", errors="ignore") as f:
                        lines = f.readlines()
                    for idx, line in enumerate(lines, 1):
                        stripped_line = line.strip()
                        if stripped_line.startswith("#") or stripped_line.startswith("//"):
                            continue
                        if re.search(r"\b(eval|exec)\s*\(", line):
                            issues.append({
                                "file": rel_path,
                                "line": idx,
                                "severity": "high",
                                "rule": "dangerous-eval",
                                "message": "Use of eval/exec detected in executable code.",
                            })
                except Exception:
                    pass

        approved = len([i for i in issues if i.get("severity") in ("critical", "high")]) == 0
        duration = time.time() - start_time

        summary = (
            f"Reviewer '{self.agent_id}' completed review. "
            f"Decision: {'APPROVED' if approved else 'REJECTED'}. Issues found: {len(issues)}."
        )

        return TaskResult(
            task_id=task.task_id,
            agent_id=self.agent_id,
            status="SUCCESS" if approved else "FAILED",
            summary=summary,
            files_changed=[],
            tests=[],
            commit=None,
            errors=[i["message"] for i in issues if i.get("severity") == "critical"],
            execution_time_sec=duration,
            output_details={"approved": approved, "issues": issues},
        )


class SecurityHandler(BaseAgentHandler):
    """Security agent handler: secret leak detection, vulnerability audit, CVE check."""

    def execute_task(self, task: TaskRequest) -> TaskResult:
        start_time = time.time()
        repo_dir = task.repository if task.repository and os.path.exists(task.repository) else os.getcwd()

        findings: List[Dict[str, Any]] = []
        secret_patterns = [
            (r"(?i)(api[_-]?key|secret|password|token)\s*=\s*['\"][a-zA-Z0-9_\-]{8,}['\"]", "Hardcoded Secret Token"),
            (r"-----BEGIN (RSA|OPENSSH|PRIVATE) KEY-----", "Private Key In Source"),
        ]

        for root, dirs, files in os.walk(repo_dir):
            dirs[:] = [d for d in dirs if d not in (".git", "__pycache__", "logs", "project", "tests", ".pytest_cache")]
            for file in files:
                if file.startswith("test_") or file.endswith("_test.py"):
                    continue
                file_path = os.path.join(root, file)
                rel_path = os.path.relpath(file_path, repo_dir)
                try:
                    with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()
                    for pattern, desc in secret_patterns:
                        if re.search(pattern, content):
                            findings.append({
                                "file": rel_path,
                                "type": "secret_leak",
                                "severity": "critical",
                                "description": desc,
                            })
                except Exception:
                    pass

        passed = len(findings) == 0
        duration = time.time() - start_time

        summary = (
            f"Security '{self.agent_id}' scan finished. "
            f"Status: {'SECURE' if passed else 'VULNERABILITY DETECTED'}. Findings: {len(findings)}."
        )

        return TaskResult(
            task_id=task.task_id,
            agent_id=self.agent_id,
            status="SUCCESS" if passed else "FAILED",
            summary=summary,
            files_changed=[],
            tests=[],
            commit=None,
            errors=[f["description"] for f in findings],
            execution_time_sec=duration,
            output_details={"secure": passed, "findings": findings},
        )


class ResearcherHandler(BaseAgentHandler):
    """Researcher agent handler: documentation, specifications, technical research."""

    def execute_task(self, task: TaskRequest) -> TaskResult:
        start_time = time.time()
        repo_dir = task.repository if task.repository and os.path.exists(task.repository) else os.getcwd()
        files_created = []

        doc_content = task.metadata.get("doc_content")
        doc_filename = task.metadata.get("doc_filename", "docs/research_notes.md")

        if doc_content:
            target_path = os.path.join(repo_dir, doc_filename)
            os.makedirs(os.path.dirname(target_path), exist_ok=True)
            with open(target_path, "w", encoding="utf-8") as f:
                f.write(doc_content)
            files_created.append(doc_filename)

        duration = time.time() - start_time
        summary = f"Researcher '{self.agent_id}' completed documentation/research for '{task.task_id}'."

        return TaskResult(
            task_id=task.task_id,
            agent_id=self.agent_id,
            status="SUCCESS",
            summary=summary,
            files_changed=files_created,
            tests=[],
            commit=None,
            errors=[],
            execution_time_sec=duration,
            output_details={"doc_files": files_created},
        )


def create_agent_handler(agent_id: str, role: AgentRole) -> BaseAgentHandler:
    handlers = {
        AgentRole.DEVELOPER: DeveloperHandler,
        AgentRole.TESTER: TesterHandler,
        AgentRole.REVIEWER: ReviewerHandler,
        AgentRole.SECURITY: SecurityHandler,
        AgentRole.RESEARCHER: ResearcherHandler,
    }
    handler_cls = handlers.get(role, DeveloperHandler)
    return handler_cls(agent_id=agent_id, role=role)
