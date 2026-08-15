//! External tool diagnostics for `mag-manager` (envdoctor & jpcargo).

use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SystemDiagnosticReport {
    pub is_healthy: bool,
    pub tools: HashMap<String, bool>,
    pub issues: Vec<String>,
}

pub struct EnvDoctor;

impl EnvDoctor {
    pub fn diagnose() -> SystemDiagnosticReport {
        let mut tools = HashMap::new();
        let mut issues = Vec::new();

        let checked = ["cargo", "rustc", "git", "docker", "python3"];
        for tool in checked {
            let installed = Command::new("which")
                .arg(tool)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            tools.insert(tool.to_string(), installed);
            if !installed && (tool == "cargo" || tool == "git") {
                issues.push(format!("Required tool '{}' is missing in PATH", tool));
            }
        }

        let is_healthy = issues.is_empty();
        SystemDiagnosticReport {
            is_healthy,
            tools,
            issues,
        }
    }
}

pub struct JpCargoAnalyzer;

impl JpCargoAnalyzer {
    pub fn explain_rust_error(stderr_output: &str) -> Option<&'static str> {
        if stderr_output.contains("cannot find value") {
            Some("変数または関数が見つかりません。スコープまたはuse宣言を確認してください。")
        } else if stderr_output.contains("borrowed as mutable") {
            Some("ミュータブル（可変）借用の衝突が発生しています。")
        } else if stderr_output.contains("does not live long enough") {
            Some("ライフタイムが短すぎます。参照先の値が破棄された後も参照を保持しようとしています。")
        } else if stderr_output.contains("mismatched types") {
            Some("型の不一致です。期待される型と実際の型を確認してください。")
        } else {
            None
        }
    }
}
