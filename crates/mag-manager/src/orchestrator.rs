//! Multi-Agent Orchestration coordinator in Rust.

use crate::evaluator::{EvaluationVerdict, ResultEvaluator};
use mag_agent::AgentDefinition;
use mag_common::{AgentRole, TaskRequest, TaskResult};
use mag_config::ProjectConfig;
use mag_git::GitManager;
use mag_scheduler::TaskScheduler;
use mag_storage::Storage;
use mag_task::Task;
use mag_worker::execute_task_for_agent;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Storage error: {0}")]
    Storage(#[from] mag_storage::StorageError),
    #[error("Git error: {0}")]
    Git(#[from] mag_git::GitError),
    #[error("Task execution failed permanently: {0}")]
    Fatal(String),
}

pub struct Orchestrator {
    pub config: ProjectConfig,
    pub storage: Storage,
    pub git: GitManager,
    pub evaluator: ResultEvaluator,
    pub repo_path: PathBuf,
}

impl Orchestrator {
    pub fn new<P: AsRef<Path>>(repo_path: P, db_path: P) -> Result<Self, OrchestratorError> {
        let repo_buf = repo_path.as_ref().to_path_buf();
        let storage = Storage::new(db_path)?;
        let git = GitManager::new(&repo_buf);
        let config = ProjectConfig::default_project("mag-project");
        let evaluator = ResultEvaluator::new(config.manager.max_retries);

        Ok(Self {
            config,
            storage,
            git,
            evaluator,
            repo_path: repo_buf,
        })
    }

    pub fn decompose_requirement(&self, prompt: &str) -> Result<Vec<Task>, OrchestratorError> {
        let existing = self.storage.list_tasks()?;
        let base_num = existing.len() + 1;

        let id1 = format!("TASK-{:03}", base_num);
        let id2 = format!("TASK-{:03}", base_num + 1);
        let id3 = format!("TASK-{:03}", base_num + 2);
        let id4 = format!("TASK-{:03}", base_num + 3);
        let id5 = format!("TASK-{:03}", base_num + 4);

        let t1 = Task::new(&id1, format!("Spec: {}", prompt), format!("Architecture notes for: {}", prompt), "agent-e", AgentRole::Researcher);
        let t2 = Task::new(&id2, format!("Implementation: {}", prompt), format!("Implement source for: {}", prompt), "agent-a", AgentRole::Developer)
            .with_dependencies(vec![id1]);
        let t3 = Task::new(&id3, format!("Testing: {}", prompt), format!("Run tests for: {}", prompt), "agent-b", AgentRole::Tester)
            .with_dependencies(vec![id2]);
        let t4 = Task::new(&id4, format!("Review: {}", prompt), format!("Review code for: {}", prompt), "agent-c", AgentRole::Reviewer)
            .with_dependencies(vec![id3]);
        let t5 = Task::new(&id5, format!("Security: {}", prompt), format!("Security audit for: {}", prompt), "agent-d", AgentRole::Security)
            .with_dependencies(vec![id4]);

        let tasks = vec![t1, t2, t3, t4, t5];
        for t in &tasks {
            self.storage.save_task(t)?;
        }

        Ok(tasks)
    }

    pub fn execute_task_locally(&self, task: &mut Task, metadata: Option<HashMap<String, serde_json::Value>>) -> Result<TaskResult, OrchestratorError> {
        let agent = AgentDefinition::default_for_role(task.role);
        let task_req = TaskRequest {
            task_id: task.id.clone(),
            r#type: task.role.to_string(),
            title: task.title.clone(),
            description: task.description.clone(),
            repository: self.repo_path.to_string_lossy().to_string(),
            branch: format!("{}/{}", task.assigned_agent, task.id.to_lowercase()),
            timeout_seconds: 300,
            context_files: vec![],
            metadata: metadata.unwrap_or_default(),
        };

        let result = execute_task_for_agent(&agent, &task_req);
        let (verdict, next_status) = self.evaluator.evaluate(task, &result);

        task.status = next_status;
        task.result = Some(result.clone());
        if matches!(verdict, EvaluationVerdict::Retry { .. }) {
            task.retry_count += 1;
        }

        self.storage.save_task(task)?;
        Ok(result)
    }

    pub fn run_orchestration_loop(&self, max_iterations: usize) -> Result<bool, OrchestratorError> {
        let mut iterations = 0;

        while iterations < max_iterations {
            iterations += 1;
            let all_tasks = self.storage.list_tasks()?;
            let ready_tasks = TaskScheduler::get_ready_tasks(&all_tasks);

            if ready_tasks.is_empty() {
                break;
            }

            for mut task in ready_tasks {
                println!("[*] Dispatching task [{}] '{}' to [{}]...", task.id, task.title, task.assigned_agent);
                let res = self.execute_task_locally(&mut task, None)?;
                println!("    -> Result: {} | {}", res.status, res.summary);
            }
        }

        let final_tasks = self.storage.list_tasks()?;
        Ok(TaskScheduler::is_all_completed(&final_tasks))
    }
}
