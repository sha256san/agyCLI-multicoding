//! Worker agent execution engine for `mag`.

pub mod executor;
pub mod handlers;

pub use executor::{CommandExecutor, ExecutionOutput};
pub use handlers::execute_task_for_agent;

#[cfg(test)]
mod tests {
    use super::*;
    use mag_agent::AgentDefinition;
    use mag_common::{AgentRole, TaskRequest};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_worker_developer_handler() {
        let dir = tempdir().unwrap();
        let agent = AgentDefinition::default_for_role(AgentRole::Developer);

        let mut meta = HashMap::new();
        meta.insert(
            "files".into(),
            serde_json::json!({
                "hello.rs": "fn main() { println!(\"Hello!\"); }\n"
            }),
        );

        let task = TaskRequest {
            task_id: "T1".into(),
            r#type: "developer".into(),
            title: "Write hello".into(),
            description: "Hello program".into(),
            repository: dir.path().to_string_lossy().to_string(),
            branch: "main".into(),
            timeout_seconds: 300,
            context_files: vec![],
            metadata: meta,
        };

        let result = execute_task_for_agent(&agent, &task);
        assert!(result.is_success());
        assert!(dir.path().join("hello.rs").exists());
    }
}
