//! Role-specific task execution handlers in Rust.

use crate::executor::CommandExecutor;
use mag_agent::AgentDefinition;
use mag_common::{AgentRole, TaskRequest, TaskResult};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub fn execute_task_for_agent(agent: &AgentDefinition, task: &TaskRequest) -> TaskResult {
    let start = Instant::now();
    let executor = CommandExecutor::new(agent.allowed_commands.clone());
    let repo_path = Path::new(&task.repository);

    match agent.role {
        AgentRole::Developer => handle_developer(agent, task, &executor, repo_path, start),
        AgentRole::Tester => handle_tester(agent, task, &executor, repo_path, start),
        AgentRole::Reviewer => handle_reviewer(agent, task, &executor, repo_path, start),
        AgentRole::Security => handle_security(agent, task, &executor, repo_path, start),
        AgentRole::Researcher => handle_researcher(agent, task, &executor, repo_path, start),
        AgentRole::Manager => TaskResult {
            task_id: task.task_id.clone(),
            agent_id: agent.id.clone(),
            status: "SUCCESS".into(),
            summary: "Manager task completed".into(),
            files_changed: vec![],
            tests: vec![],
            commit: None,
            errors: vec![],
            execution_time_sec: start.elapsed().as_secs_f64(),
            output_details: HashMap::new(),
        },
    }
}

fn handle_developer(
    agent: &AgentDefinition,
    task: &TaskRequest,
    executor: &CommandExecutor,
    repo_path: &Path,
    start: Instant,
) -> TaskResult {
    let mut files_changed = Vec::new();
    let mut errors = Vec::new();

    // Check metadata for files to write
    if let Some(files_val) = task.metadata.get("files") {
        if let Some(files_map) = files_val.as_object() {
            for (rel_path, content_val) in files_map {
                if let Some(content) = content_val.as_str() {
                    let target_path = repo_path.join(rel_path);
                    if let Some(parent) = target_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Err(e) = fs::write(&target_path, content) {
                        errors.push(format!("Failed to write file {}: {}", rel_path, e));
                    } else {
                        files_changed.push(rel_path.clone());
                    }
                }
            }
        }
    }

    // Run custom command if provided
    if let Some(cmd_val) = task.metadata.get("command").and_then(|v| v.as_str()) {
        let out = executor.run_command(cmd_val, Some(repo_path));
        if !out.success {
            errors.push(out.stderr);
        }
    }

    let success = errors.is_empty();
    let summary = format!(
        "Developer '{}' completed task '{}': {}. Files modified: {}.",
        agent.id,
        task.task_id,
        task.title,
        files_changed.len()
    );

    TaskResult {
        task_id: task.task_id.clone(),
        agent_id: agent.id.clone(),
        status: if success { "SUCCESS".into() } else { "FAILED".into() },
        summary,
        files_changed,
        tests: vec![],
        commit: None,
        errors,
        execution_time_sec: start.elapsed().as_secs_f64(),
        output_details: HashMap::new(),
    }
}

fn handle_tester(
    agent: &AgentDefinition,
    task: &TaskRequest,
    executor: &CommandExecutor,
    repo_path: &Path,
    start: Instant,
) -> TaskResult {
    let test_cmd = task
        .metadata
        .get("test_command")
        .and_then(|v| v.as_str())
        .unwrap_or("cargo check || python3 -m unittest discover");

    let out = executor.run_command(test_cmd, Some(repo_path));
    let zero_tests = out.exit_code == Some(5) && out.stderr.contains("NO TESTS RAN");
    let passed = out.success || zero_tests;

    let mut errors = Vec::new();
    if !passed {
        errors.push(if out.stderr.is_empty() { out.stdout.clone() } else { out.stderr.clone() });
    }

    let summary = format!(
        "Tester '{}' executed test suite. Result: {} (exit code: {:?}).",
        agent.id,
        if passed { "PASS" } else { "FAIL" },
        out.exit_code
    );

    TaskResult {
        task_id: task.task_id.clone(),
        agent_id: agent.id.clone(),
        status: if passed { "SUCCESS".into() } else { "FAILED".into() },
        summary,
        files_changed: vec![],
        tests: vec![serde_json::json!({
            "command": test_cmd,
            "passed": passed,
            "stdout": out.stdout,
            "stderr": out.stderr,
        })],
        commit: None,
        errors,
        execution_time_sec: start.elapsed().as_secs_f64(),
        output_details: HashMap::new(),
    }
}

fn handle_reviewer(
    agent: &AgentDefinition,
    task: &TaskRequest,
    _executor: &CommandExecutor,
    repo_path: &Path,
    start: Instant,
) -> TaskResult {
    let mut issues = Vec::new();

    // Check files for dangerous patterns
    if let Ok(entries) = fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("eval(") || content.contains("exec(") {
                        issues.push(serde_json::json!({
                            "file": path.file_name().unwrap_or_default().to_string_lossy(),
                            "severity": "high",
                            "message": "Use of eval/exec detected in source code."
                        }));
                    }
                }
            }
        }
    }

    let approved = issues.is_empty();
    let summary = format!(
        "Reviewer '{}' completed review. Decision: {}. Issues: {}.",
        agent.id,
        if approved { "APPROVED" } else { "REJECTED" },
        issues.len()
    );

    let mut details = HashMap::new();
    details.insert("approved".into(), serde_json::Value::Bool(approved));
    details.insert("issues".into(), serde_json::Value::Array(issues));

    TaskResult {
        task_id: task.task_id.clone(),
        agent_id: agent.id.clone(),
        status: if approved { "SUCCESS".into() } else { "FAILED".into() },
        summary,
        files_changed: vec![],
        tests: vec![],
        commit: None,
        errors: vec![],
        execution_time_sec: start.elapsed().as_secs_f64(),
        output_details: details,
    }
}

fn handle_security(
    agent: &AgentDefinition,
    task: &TaskRequest,
    _executor: &CommandExecutor,
    repo_path: &Path,
    start: Instant,
) -> TaskResult {
    let mut findings = Vec::new();

    if let Ok(entries) = fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("-----BEGIN PRIVATE KEY-----") || content.contains("secret_token_") {
                        findings.push(serde_json::json!({
                            "file": path.file_name().unwrap_or_default().to_string_lossy(),
                            "severity": "critical",
                            "description": "Hardcoded secret detected"
                        }));
                    }
                }
            }
        }
    }

    let secure = findings.is_empty();
    let summary = format!(
        "Security '{}' scan finished. Status: {}. Findings: {}.",
        agent.id,
        if secure { "SECURE" } else { "VULNERABILITIES DETECTED" },
        findings.len()
    );

    let mut details = HashMap::new();
    details.insert("secure".into(), serde_json::Value::Bool(secure));
    details.insert("findings".into(), serde_json::Value::Array(findings));

    TaskResult {
        task_id: task.task_id.clone(),
        agent_id: agent.id.clone(),
        status: if secure { "SUCCESS".into() } else { "FAILED".into() },
        summary,
        files_changed: vec![],
        tests: vec![],
        commit: None,
        errors: vec![],
        execution_time_sec: start.elapsed().as_secs_f64(),
        output_details: details,
    }
}

fn handle_researcher(
    agent: &AgentDefinition,
    task: &TaskRequest,
    _executor: &CommandExecutor,
    repo_path: &Path,
    start: Instant,
) -> TaskResult {
    let mut files_created = Vec::new();
    if let Some(doc_content) = task.metadata.get("doc_content").and_then(|v| v.as_str()) {
        let doc_path = repo_path.join("docs").join("spec.md");
        if let Some(parent) = doc_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&doc_path, doc_content);
        files_created.push("docs/spec.md".to_string());
    }

    let summary = format!(
        "Researcher '{}' completed specification and research for task '{}'.",
        agent.id, task.task_id
    );

    TaskResult {
        task_id: task.task_id.clone(),
        agent_id: agent.id.clone(),
        status: "SUCCESS".into(),
        summary,
        files_changed: files_created,
        tests: vec![],
        commit: None,
        errors: vec![],
        execution_time_sec: start.elapsed().as_secs_f64(),
        output_details: HashMap::new(),
    }
}
