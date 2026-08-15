//! Task data models and lifecycle management for `mag`.

use chrono::{DateTime, Utc};
use mag_common::{AgentRole, TaskResult, TaskStatus};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TaskError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
    #[error("Missing task field: {0}")]
    MissingField(String),
}

/// Task item in the orchestration system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub assigned_agent: String,
    pub role: AgentRole,
    pub priority: String, // "low", "medium", "high"
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub result: Option<TaskResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        assigned_agent: impl Into<String>,
        role: AgentRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            assigned_agent: assigned_agent.into(),
            role,
            priority: "medium".into(),
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            retry_count: 0,
            max_retries: 3,
            result: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn transition_to(&mut self, next_status: TaskStatus) -> Result<(), TaskError> {
        // Validate valid transitions
        let valid = match (self.status, next_status) {
            (TaskStatus::Pending, TaskStatus::Assigned) => true,
            (TaskStatus::Pending, TaskStatus::Running) => true,
            (TaskStatus::Assigned, TaskStatus::Running) => true,
            (TaskStatus::Running, TaskStatus::Review) => true,
            (TaskStatus::Running, TaskStatus::Testing) => true,
            (TaskStatus::Running, TaskStatus::Completed) => true,
            (TaskStatus::Running, TaskStatus::Failed) => true,
            (TaskStatus::Running, TaskStatus::Retrying) => true,
            (TaskStatus::Review, TaskStatus::Completed) => true,
            (TaskStatus::Review, TaskStatus::Retrying) => true,
            (TaskStatus::Review, TaskStatus::Failed) => true,
            (TaskStatus::Testing, TaskStatus::Completed) => true,
            (TaskStatus::Testing, TaskStatus::Retrying) => true,
            (TaskStatus::Testing, TaskStatus::Failed) => true,
            (TaskStatus::Retrying, TaskStatus::Running) => true,
            (TaskStatus::Failed, TaskStatus::Retrying) => true,
            (TaskStatus::Failed, TaskStatus::FailedPermanently) => true,
            (TaskStatus::Retrying, TaskStatus::FailedPermanently) => true,
            (_, TaskStatus::Cancelled) => true,
            (from, to) if from == to => true,
            _ => false,
        };

        if valid {
            self.status = next_status;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(TaskError::InvalidStateTransition {
                from: self.status.to_string(),
                to: next_status.to_string(),
            })
        }
    }

    pub fn is_ready(&self, completed_task_ids: &[String]) -> bool {
        (self.status == TaskStatus::Pending || self.status == TaskStatus::Retrying)
            && self.dependencies.iter().all(|dep| completed_task_ids.contains(dep))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_lifecycle_transitions() {
        let mut task = Task::new("TASK-01", "Init", "Init task", "agent-a", AgentRole::Developer);
        assert_eq!(task.status, TaskStatus::Pending);

        task.transition_to(TaskStatus::Running).unwrap();
        assert_eq!(task.status, TaskStatus::Running);

        task.transition_to(TaskStatus::Completed).unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn test_task_dependency_ready() {
        let task = Task::new("TASK-02", "Child", "Child task", "agent-b", AgentRole::Tester)
            .with_dependencies(vec!["TASK-01".into()]);

        assert!(!task.is_ready(&[]));
        assert!(task.is_ready(&["TASK-01".into()]));
    }
}
