//! Task scheduler, collaborative multi-role dispatcher, and DAG dependency resolver for `mag`.

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

    /// Assign tasks to available workers collaboratively based on worker pool count.
    /// When worker_count is small (e.g. 2), workers share multiple roles dynamically.
    pub fn assign_collaborative_workers(tasks: &mut [Task], worker_count: usize) {
        let count = worker_count.max(1);
        for (i, task) in tasks.iter_mut().enumerate() {
            let worker_idx = (i % count) + 1;
            task.assigned_agent = format!("agent-{}", worker_idx);
        }
    }

    /// Dynamic work-stealing: finds the next available ready task for an idle worker.
    pub fn claim_next_task_for_worker(tasks: &mut [Task], worker_id: &str) -> Option<Task> {
        let completed_ids: HashSet<String> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id.clone())
            .collect();

        for task in tasks.iter_mut() {
            if (task.status == TaskStatus::Pending || task.status == TaskStatus::Retrying)
                && task.dependencies.iter().all(|dep| completed_ids.contains(dep))
            {
                task.assigned_agent = worker_id.to_string();
                task.status = TaskStatus::Running;
                return Some(task.clone());
            }
        }
        None
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

    #[test]
    fn test_collaborative_two_workers() {
        let t1 = Task::new("T1", "Spec", "Spec", "agent-e", AgentRole::Researcher);
        let t2 = Task::new("T2", "Impl", "Impl", "agent-a", AgentRole::Developer);
        let t3 = Task::new("T3", "Test", "Test", "agent-b", AgentRole::Tester);
        let mut tasks = vec![t1, t2, t3];

        TaskScheduler::assign_collaborative_workers(&mut tasks, 2);
        assert_eq!(tasks[0].assigned_agent, "agent-1");
        assert_eq!(tasks[1].assigned_agent, "agent-2");
        assert_eq!(tasks[2].assigned_agent, "agent-1"); // wraps around to agent-1
    }
}
