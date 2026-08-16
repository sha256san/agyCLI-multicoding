//! Common types, enums, models, and constants for `mag`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent specialization roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    Manager,
    Developer,
    Tester,
    Reviewer,
    Security,
    Researcher,
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::Manager => write!(f, "manager"),
            AgentRole::Developer => write!(f, "developer"),
            AgentRole::Tester => write!(f, "tester"),
            AgentRole::Reviewer => write!(f, "reviewer"),
            AgentRole::Security => write!(f, "security"),
            AgentRole::Researcher => write!(f, "researcher"),
        }
    }
}

/// Task execution state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Pending,
    Assigned,
    Running,
    Waiting,
    Review,
    Testing,
    Failed,
    Retrying,
    Completed,
    Cancelled,
    FailedPermanently,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "PENDING"),
            TaskStatus::Assigned => write!(f, "ASSIGNED"),
            TaskStatus::Running => write!(f, "RUNNING"),
            TaskStatus::Waiting => write!(f, "WAITING"),
            TaskStatus::Review => write!(f, "REVIEW"),
            TaskStatus::Testing => write!(f, "TESTING"),
            TaskStatus::Failed => write!(f, "FAILED"),
            TaskStatus::Retrying => write!(f, "RETRYING"),
            TaskStatus::Completed => write!(f, "COMPLETED"),
            TaskStatus::Cancelled => write!(f, "CANCELLED"),
            TaskStatus::FailedPermanently => write!(f, "FAILED_PERMANENTLY"),
        }
    }
}

/// Agent execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentExecutionStatus {
    Idle,
    Running,
    Error,
    Stopped,
}

/// Review or security finding item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueItem {
    pub severity: String,
    pub file: String,
    pub line: Option<usize>,
    pub rule: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Task Request sent to Worker API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_id: String,
    pub r#type: String,
    pub title: String,
    pub description: String,
    pub repository: String,
    pub branch: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub context_files: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_timeout() -> u64 {
    300
}

/// Structured Task Result returned from Worker API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: String,
    pub status: String, // "SUCCESS" | "FAILED"
    pub summary: String,
    #[serde(default)]
    pub files_changed: Vec<String>,
    #[serde(default)]
    pub tests: Vec<serde_json::Value>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub execution_time_sec: f64,
    #[serde(default)]
    pub output_details: HashMap<String, serde_json::Value>,
}

impl TaskResult {
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("SUCCESS") || self.status.eq_ignore_ascii_case("COMPLETED")
    }
}

/// Agent runtime status info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub role: AgentRole,
    pub status: AgentExecutionStatus,
    pub current_task_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub progress_percent: u32,
    pub host: String,
    pub port: u16,
}

/// Authenticated user identity info.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthUser {
    pub provider: String, // "google" | "gemini" | "token"
    pub email: Option<String>,
    pub name: Option<String>,
    pub id: String,
}

/// Agent authentication status states specified in addplan7.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentAuthState {
    Uninitialized,
    Authenticating,
    Authenticated,
    Expired,
    AuthError,
}

impl std::fmt::Display for AgentAuthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentAuthState::Uninitialized => write!(f, "UNINITIALIZED"),
            AgentAuthState::Authenticating => write!(f, "AUTHENTICATING"),
            AgentAuthState::Authenticated => write!(f, "AUTHENTICATED"),
            AgentAuthState::Expired => write!(f, "EXPIRED"),
            AgentAuthState::AuthError => write!(f, "AUTH_ERROR"),
        }
    }
}

/// Stored authentication tokens and expiration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Authentication state configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub user: Option<AuthUser>,
    pub token: Option<AuthToken>,
    pub updated_at: DateTime<Utc>,
}

impl AuthConfig {
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some() && self.token.is_some()
    }
}

/// Dynamic Worker Pool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolConfig {
    #[serde(default = "default_min_workers")]
    pub min_workers: usize,
    #[serde(default = "default_max_workers")]
    pub max_workers: usize,
    #[serde(default = "default_current_workers")]
    pub current_workers: usize,
    #[serde(default = "default_auto_scale")]
    pub auto_scale: bool,
}

fn default_min_workers() -> usize { 1 }
fn default_max_workers() -> usize { 10 }
fn default_current_workers() -> usize { 5 }
fn default_auto_scale() -> bool { true }

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            min_workers: default_min_workers(),
            max_workers: default_max_workers(),
            current_workers: default_current_workers(),
            auto_scale: default_auto_scale(),
        }
    }
}

pub mod constants {
    pub const DEFAULT_MANAGER_PORT: u16 = 8000;
    pub const DEFAULT_MAX_RETRY: u32 = 3;
    pub const DEFAULT_TASK_TIMEOUT_SECONDS: u64 = 300;

    pub const DANGEROUS_COMMAND_KEYWORDS: &[&str] = &[
        "rm -rf", "mkfs", "dd if=", ":(){ :|:& };:", "chmod -R 777",
        "sudo", "shutdown", "reboot", "iptables -F"
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_request_serialization() {
        let req = TaskRequest {
            task_id: "TASK-001".into(),
            r#type: "developer".into(),
            title: "Build CLI".into(),
            description: "Implement CLI".into(),
            repository: "/workspace".into(),
            branch: "main".into(),
            timeout_seconds: 300,
            context_files: vec!["src/main.rs".into()],
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: TaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_id, "TASK-001");
        assert_eq!(deserialized.context_files, vec!["src/main.rs"]);
    }
}
