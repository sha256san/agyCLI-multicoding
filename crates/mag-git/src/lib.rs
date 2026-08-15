//! Git version control and worktree management for `mag`.

use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git command failed: {command} (exit code: {exit_code:?})\nStderr: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct GitManager {
    repo_path: PathBuf,
}

impl GitManager {
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    fn run_git(&self, args: &[&str], cwd: Option<&Path>) -> Result<String, GitError> {
        let work_dir = cwd.unwrap_or(&self.repo_path);
        let output = Command::new("git")
            .args(args)
            .current_dir(work_dir)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(GitError::CommandFailed {
                command: format!("git {}", args.join(" ")),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            })
        }
    }

    pub fn is_repo(&self) -> bool {
        self.run_git(&["rev-parse", "--is-inside-work-tree"], None).is_ok()
    }

    pub fn init_repo(&self) -> Result<(), GitError> {
        if !self.is_repo() {
            self.run_git(&["init"], None)?;
            self.run_git(&["config", "user.name", "mag-manager"], None)?;
            self.run_git(&["config", "user.email", "mag@antigravity.local"], None)?;
        }
        Ok(())
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<(), GitError> {
        self.run_git(&["checkout", "-B", branch_name], None)?;
        Ok(())
    }

    pub fn create_worktree(&self, branch: &str, path: &Path) -> Result<(), GitError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let path_str = path.to_string_lossy();
        self.run_git(&["worktree", "add", "-B", branch, &path_str, "main"], None)?;
        Ok(())
    }

    pub fn remove_worktree(&self, path: &Path) -> Result<(), GitError> {
        let path_str = path.to_string_lossy();
        self.run_git(&["worktree", "remove", "--force", &path_str], None)?;
        Ok(())
    }

    pub fn get_diff(&self, base_branch: &str, target_branch: Option<&str>) -> Result<String, GitError> {
        let target = target_branch.unwrap_or("HEAD");
        self.run_git(&["diff", &format!("{}...{}", base_branch, target)], None)
    }

    pub fn merge_branch(&self, branch_name: &str, target_branch: &str) -> Result<String, GitError> {
        self.run_git(&["checkout", target_branch], None)?;
        self.run_git(&["merge", branch_name], None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_git_init() {
        let dir = tempdir().unwrap();
        let git = GitManager::new(dir.path());
        assert!(!git.is_repo());
        git.init_repo().unwrap();
        assert!(git.is_repo());
    }
}
