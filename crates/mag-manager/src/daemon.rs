//! Daemon lifecycle management, PID tracking, and crash recovery for `mag`.

use chrono::{DateTime, Utc};
use mag_storage::Storage;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub pid: u32,
    pub status: String, // "RUNNING" | "STOPPED"
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub active_tasks_count: usize,
}

pub struct DaemonManager {
    root_path: PathBuf,
}

impl DaemonManager {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    fn pid_file(&self) -> PathBuf {
        self.root_path.join(".mag").join("manager.pid")
    }

    fn info_file(&self) -> PathBuf {
        self.root_path.join(".mag").join("daemon.json")
    }

    pub fn is_running(&self) -> bool {
        let pid_file = self.pid_file();
        if !pid_file.exists() {
            return false;
        }

        if let Ok(pid_str) = fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                // Check if PID is alive using libc kill(pid, 0)
                unsafe {
                    return libc::kill(pid, 0) == 0;
                }
            }
        }
        false
    }

    pub fn start_daemon(&self, storage: &Storage) -> anyhow::Result<DaemonInfo> {
        let mag_dir = self.root_path.join(".mag");
        fs::create_dir_all(&mag_dir)?;

        let pid = std::process::id();
        fs::write(self.pid_file(), pid.to_string())?;

        // Perform crash recovery on any tasks stuck in RUNNING state
        self.recover_crashed_tasks(storage)?;

        let all_tasks = storage.list_tasks()?;
        let active_tasks = all_tasks
            .iter()
            .filter(|t| t.status.to_string() == "RUNNING" || t.status.to_string() == "PENDING")
            .count();

        let info = DaemonInfo {
            pid,
            status: "RUNNING".to_string(),
            started_at: Utc::now(),
            uptime_seconds: 0,
            active_tasks_count: active_tasks,
        };

        let json = serde_json::to_string_pretty(&info)?;
        fs::write(self.info_file(), json)?;

        Ok(info)
    }

    pub fn stop_daemon(&self) -> anyhow::Result<()> {
        let pid_file = self.pid_file();
        if pid_file.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    unsafe {
                        let _ = libc::kill(pid, libc::SIGTERM);
                    }
                }
            }
            let _ = fs::remove_file(&pid_file);
        }

        if self.info_file().exists() {
            let _ = fs::remove_file(self.info_file());
        }

        Ok(())
    }

    pub fn get_status(&self, storage: Option<&Storage>) -> DaemonInfo {
        let is_alive = self.is_running();
        let active_tasks = if let Some(st) = storage {
            st.list_tasks()
                .map(|ts| ts.iter().filter(|t| t.status.to_string() == "RUNNING").count())
                .unwrap_or(0)
        } else {
            0
        };

        if is_alive {
            if let Ok(content) = fs::read_to_string(self.info_file()) {
                if let Ok(mut info) = serde_json::from_str::<DaemonInfo>(&content) {
                    info.uptime_seconds = Utc::now().signed_duration_since(info.started_at).num_seconds().max(0) as u64;
                    info.active_tasks_count = active_tasks;
                    return info;
                }
            }

            let pid_str = fs::read_to_string(self.pid_file()).unwrap_or_default();
            let pid = pid_str.trim().parse::<u32>().unwrap_or(0);
            DaemonInfo {
                pid,
                status: "RUNNING".to_string(),
                started_at: Utc::now(),
                uptime_seconds: 0,
                active_tasks_count: active_tasks,
            }
        } else {
            DaemonInfo {
                pid: 0,
                status: "STOPPED".to_string(),
                started_at: Utc::now(),
                uptime_seconds: 0,
                active_tasks_count: 0,
            }
        }
    }

    pub fn recover_crashed_tasks(&self, storage: &Storage) -> anyhow::Result<usize> {
        let tasks = storage.list_tasks()?;
        let mut recovered = 0;

        for mut task in tasks {
            if task.status.to_string() == "RUNNING" {
                task.status = mag_common::TaskStatus::Pending;
                storage.save_task(&task)?;
                storage.record_event(&task.id, &task.assigned_agent, "TASK_RECOVERED", &"Task recovered from crash state")?;
                recovered += 1;
            }
        }

        Ok(recovered)
    }
}
