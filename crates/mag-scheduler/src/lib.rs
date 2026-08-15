//! Task scheduler and DAG dependency resolver for `mag`.

use mag_common::TaskStatus;
use mag_task::Task;
use std::collections::HashSet;

pub struct TaskScheduler;

impl TaskScheduler {
    pub fn get_ready_tasks(tasks: &[Task]) -> Vec<Task> {
        let completed_ids: HashSet<String> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id.clone())
            .collect();

        tasks
            .iter()
            .filter(|t| {
                (t.status == TaskStatus::Pending || t.status == TaskStatus::Retrying)
                    && t.dependencies.iter().all(|dep| completed_ids.contains(dep))
            })
            .cloned()
            .collect()
    }

    pub fn is_all_completed(tasks: &[Task]) -> bool {
        if tasks.is_empty() {
            return true;
        }
        tasks.iter().all(|t| t.status == TaskStatus::Completed)
    }

    pub fn has_permanently_failed(tasks: &[Task]) -> bool {
        tasks.iter().any(|t| t.status == TaskStatus::FailedPermanently)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mag_common::AgentRole;

    #[test]
    fn test_scheduler_dag() {
        let mut t1 = Task::new("T1", "Task 1", "Root", "agent-a", AgentRole::Developer);
        let t2 = Task::new("T2", "Task 2", "Child", "agent-b", AgentRole::Tester)
            .with_dependencies(vec!["T1".into()]);

        let tasks = vec![t1.clone(), t2.clone()];
        let ready = TaskScheduler::get_ready_tasks(&tasks);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "T1");

        t1.status = TaskStatus::Completed;
        let tasks_after = vec![t1, t2];
        let ready_after = TaskScheduler::get_ready_tasks(&tasks_after);
        assert_eq!(ready_after.len(), 1);
        assert_eq!(ready_after[0].id, "T2");
    }
}
