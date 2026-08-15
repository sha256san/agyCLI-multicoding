//! Result evaluation engine and self-repair coordinator for `mag-manager`.

use mag_common::{AgentRole, TaskResult, TaskStatus};
use mag_task::Task;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationVerdict {
    Pass,
    Retry { feedback: String },
    Fail { reason: String },
}

pub struct ResultEvaluator {
    max_retries: u32,
}

impl ResultEvaluator {
    pub fn new(max_retries: u32) -> Self {
        Self { max_retries }
    }

    pub fn evaluate(&self, task: &Task, result: &TaskResult) -> (EvaluationVerdict, TaskStatus) {
        if result.is_success() {
            // Additional checks for specific roles
            if task.role == AgentRole::Reviewer {
                let approved = result
                    .output_details
                    .get("approved")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if !approved {
                    return self.handle_failure(task, "Reviewer rejected changes");
                }
            }

            if task.role == AgentRole::Security {
                let secure = result
                    .output_details
                    .get("secure")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if !secure {
                    return self.handle_failure(task, "Security vulnerabilities or secret leaks detected");
                }
            }

            (EvaluationVerdict::Pass, TaskStatus::Completed)
        } else {
            let error_msg = if result.errors.is_empty() {
                result.summary.clone()
            } else {
                result.errors.join("; ")
            };
            self.handle_failure(task, &error_msg)
        }
    }

    fn handle_failure(&self, task: &Task, reason: &str) -> (EvaluationVerdict, TaskStatus) {
        if task.retry_count < self.max_retries {
            let feedback = format!(
                "Task '{}' failed (attempt {}/{}): {}",
                task.id,
                task.retry_count + 1,
                self.max_retries,
                reason
            );
            (EvaluationVerdict::Retry { feedback }, TaskStatus::Retrying)
        } else {
            let fatal_reason = format!(
                "Task '{}' exceeded maximum retries ({}): {}",
                task.id, self.max_retries, reason
            );
            (EvaluationVerdict::Fail { reason: fatal_reason }, TaskStatus::FailedPermanently)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_retry_and_max() {
        let eval = ResultEvaluator::new(3);
        let mut task = Task::new("T1", "Impl", "Impl", "agent-a", AgentRole::Developer);

        let fail_result = TaskResult {
            task_id: "T1".into(),
            agent_id: "agent-a".into(),
            status: "FAILED".into(),
            summary: "Error".into(),
            files_changed: vec![],
            tests: vec![],
            commit: None,
            errors: vec!["Syntax error".into()],
            execution_time_sec: 1.0,
            output_details: std::collections::HashMap::new(),
        };

        let (verdict, status) = eval.evaluate(&task, &fail_result);
        assert!(matches!(verdict, EvaluationVerdict::Retry { .. }));
        assert_eq!(status, TaskStatus::Retrying);

        task.retry_count = 3;
        let (verdict_max, status_max) = eval.evaluate(&task, &fail_result);
        assert!(matches!(verdict_max, EvaluationVerdict::Fail { .. }));
        assert_eq!(status_max, TaskStatus::FailedPermanently);
    }
}
