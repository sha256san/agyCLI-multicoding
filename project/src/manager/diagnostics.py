"""Environment and error diagnostic integrations (envdoctor, jpcargo)."""

import os
import shutil
import subprocess
import sys
from typing import Any, Dict, List


class EnvDoctor:
    """Environment diagnostics tool for isolating environment issues from code issues."""

    @staticmethod
    def diagnose_system() -> Dict[str, Any]:
        report = {
            "python_version": sys.version,
            "python_executable": sys.executable,
            "os_name": os.name,
            "tools": {},
            "issues": [],
        }

        checked_tools = ["git", "docker", "cargo", "rustc", "npm", "node", "python3"]
        for tool in checked_tools:
            path = shutil.which(tool)
            report["tools"][tool] = {"installed": path is not None, "path": path}

        # Check essential tools
        if not report["tools"]["git"]["installed"]:
            report["issues"].append("Git is not installed or not in PATH.")
        if not report["tools"]["python3"]["installed"]:
            report["issues"].append("Python 3 is not found in PATH.")

        report["is_healthy"] = len(report["issues"]) == 0
        return report


class JpCargoAnalyzer:
    """Rust compiler and clippy error diagnostic helper with Japanese explanations."""

    ERROR_PATTERNS = {
        "cannot find value": "変数または関数が見つかりません。スコープまたはスペル、あるいはuse宣言を確認してください。",
        "borrowed as mutable": "ミュータブル（可変）借用の衝突が発生しています。すでに不変参照または可変参照が存在します。",
        "does not live long enough": "ライフタイムが短すぎます。参照先の値が破棄された後も参照を保持しようとしています。",
        "mismatched types": "型の不一致です。期待される型と実際の型を確認してください。",
        "cannot borrow immutable local variable": "不変変数をミュータブルとして借用しようとしています。`let mut`宣言に変更してください。",
    }

    @classmethod
    def analyze_rust_error(cls, stderr_text: str) -> List[Dict[str, str]]:
        diagnoses = []
        for pattern, explanation in cls.ERROR_PATTERNS.items():
            if pattern in stderr_text:
                diagnoses.append({
                    "pattern": pattern,
                    "explanation_ja": explanation,
                })
        return diagnoses
