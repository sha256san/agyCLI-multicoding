"""Evaluation engine and self-repair coordinator for Multi-Agent Orchestrator."""

from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, List, Optional

from project.src.common.schemas import AgentRole, TaskItem, TaskResult, TaskStatus


class EvaluationVerdict(str, Enum):
    PASS = "PASS"
    RETRY = "RETRY"
    FAIL = "FAIL"


@dataclass
class EvaluationResult:
    verdict: EvaluationVerdict
    reason: str
    feedback_for_fix: Optional[str] = None
    next_status: TaskStatus = TaskStatus.COMPLETED


class ResultEvaluator:
    def __init__(self, max_retries: int = 3):
        self.max_retries = max_retries

    def evaluate_task_result(self, task: TaskItem, result: TaskResult) -> EvaluationResult:
        """Evaluate a task execution result and determine next transition."""
        if result.status == "SUCCESS":
            # For review tasks, check approved flag
            if task.role == AgentRole.REVIEWER:
                approved = result.output_details.get("approved", True)
                if not approved:
                    return self._handle_failure(
                        task,
                        result,
                        reason="Reviewer rejected code changes due to high/critical issues.",
                        feedback=f"Review feedback: {result.summary}\nIssues: {result.output_details.get('issues')}",
                    )

            # For security tasks, check secure flag
            if task.role == AgentRole.SECURITY:
                secure = result.output_details.get("secure", True)
                if not secure:
                    return self._handle_failure(
                        task,
                        result,
                        reason="Security check failed: Vulnerabilities or secret leaks detected.",
                        feedback=f"Security findings: {result.output_details.get('findings')}",
                    )

            return EvaluationResult(
                verdict=EvaluationVerdict.PASS,
                reason=f"Task '{task.task_id}' executed successfully with status SUCCESS.",
                next_status=TaskStatus.COMPLETED,
            )
        else:
            return self._handle_failure(
                task,
                result,
                reason=f"Task execution failed. Errors: {'; '.join(result.errors) if result.errors else result.summary}",
                feedback=f"Execution error summary: {result.summary}\nDetails: {result.errors}",
            )

    def _handle_failure(
        self,
        task: TaskItem,
        result: TaskResult,
        reason: str,
        feedback: str,
    ) -> EvaluationResult:
        if task.retry_count < task.max_retries:
            return EvaluationResult(
                verdict=EvaluationVerdict.RETRY,
                reason=f"{reason} (Attempt {task.retry_count + 1}/{task.max_retries})",
                feedback_for_fix=feedback,
                next_status=TaskStatus.RETRY,
            )
        else:
            return EvaluationResult(
                verdict=EvaluationVerdict.FAIL,
                reason=f"Max retries ({task.max_retries}) reached. Task permanently failed: {reason}",
                feedback_for_fix=feedback,
                next_status=TaskStatus.FAILED_PERMANENTLY,
            )
