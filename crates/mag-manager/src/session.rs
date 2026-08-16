use mag_common::AgentRole;
use mag_storage::{EventRecord, SessionRecord, Storage, StorageError};
use mag_task::Task;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageProgress {
    pub role: AgentRole,
    pub agent_id: String,
    pub percentage: u8,
    pub status: String, // "COMPLETED" | "RUNNING" | "WAITING" | "FAILED"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressSnapshot {
    pub task_id: String,
    pub status: String,
    pub overall_percentage: u8,
    pub current_step: String,
    pub stages: Vec<StageProgress>,
    pub events: Vec<EventRecord>,
}

pub struct SessionManager;

impl SessionManager {
    pub fn attach(storage: &Storage, task_id: &str) -> Result<SessionRecord, StorageError> {
        storage.attach_session(task_id)
    }

    pub fn detach(storage: &Storage, task_id: &str) -> Result<(), StorageError> {
        storage.detach_session(task_id)
    }

    pub fn get_progress(storage: &Storage, task_id: &str) -> Result<TaskProgressSnapshot, StorageError> {
        let all_tasks = storage.list_tasks()?;
        let events = storage.list_events(task_id)?;

        let mut stages = Vec::new();
        let mut completed_count = 0;
        let mut running_step = "All tasks completed".to_string();
        let mut overall_status = "PENDING".to_string();

        let task_group: Vec<&Task> = all_tasks.iter().collect();

        for t in &task_group {
            let (pct, status_str) = match t.status.to_string().as_str() {
                "COMPLETED" => {
                    completed_count += 1;
                    (100, "COMPLETED".to_string())
                }
                "RUNNING" => {
                    running_step = format!("{}: {}", t.role, t.title);
                    overall_status = "RUNNING".to_string();
                    (65, "RUNNING".to_string())
                }
                "FAILED" => {
                    overall_status = "FAILED".to_string();
                    (50, "FAILED".to_string())
                }
                _ => (0, "WAITING".to_string()),
            };

            stages.push(StageProgress {
                role: t.role,
                agent_id: t.assigned_agent.clone(),
                percentage: pct,
                status: status_str,
            });
        }

        let total_stages = if task_group.is_empty() { 1 } else { task_group.len() };
        let overall_percentage = ((completed_count as f32 / total_stages as f32) * 100.0) as u8;

        if completed_count == total_stages && total_stages > 0 {
            overall_status = "COMPLETED".to_string();
        }

        Ok(TaskProgressSnapshot {
            task_id: task_id.to_string(),
            status: overall_status,
            overall_percentage,
            current_step: running_step,
            stages,
            events,
        })
    }

    pub fn render_progress_bar(percentage: u8) -> String {
        let total_blocks: usize = 12;
        let filled = ((percentage as f32 / 100.0) * total_blocks as f32).round() as usize;
        let empty = total_blocks.saturating_sub(filled);
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }
}
