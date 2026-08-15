"""Command execution engine with safety allowlist filtering and timeout management."""

import os
import subprocess
import time
from typing import Any, Dict, List, Optional, Tuple

from project.src.common.constants import (
    DANGEROUS_COMMAND_KEYWORDS,
    ROLE_COMMAND_ALLOWLIST,
)


class CommandExecutionError(Exception):
    pass


class CommandExecutor:
    def __init__(self, role: str = "developer", allowlist: Optional[List[str]] = None):
        self.role = role
        self.allowlist = allowlist if allowlist is not None else ROLE_COMMAND_ALLOWLIST.get(role, [])

    def is_command_safe(self, command: str) -> Tuple[bool, str]:
        """Check if command is safe according to dangerous keywords and allowlist."""
        # 1. Dangerous keywords check
        for keyword in DANGEROUS_COMMAND_KEYWORDS:
            if keyword in command:
                return False, f"Command contains forbidden dangerous pattern: '{keyword}'"

        # 2. Extract base executable
        cmd_parts = command.strip().split()
        if not cmd_parts:
            return False, "Empty command"

        base_cmd = os.path.basename(cmd_parts[0])
        if self.allowlist and base_cmd not in self.allowlist:
            return False, f"Command '{base_cmd}' is not in the allowlist for role '{self.role}'"

        return True, "OK"

    def run_command(
        self,
        command: str,
        cwd: Optional[str] = None,
        timeout_seconds: int = 300,
        env: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        """Execute a shell command with safety checks and timeout."""
        is_safe, reason = self.is_command_safe(command)
        if not is_safe:
            return {
                "success": False,
                "exit_code": -1,
                "stdout": "",
                "stderr": reason,
                "duration_sec": 0.0,
                "timed_out": False,
            }

        start_time = time.time()
        try:
            process = subprocess.Popen(
                command,
                shell=True,
                cwd=cwd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                env=env or os.environ.copy(),
            )
            stdout, stderr = process.communicate(timeout=timeout_seconds)
            duration = time.time() - start_time
            return {
                "success": process.returncode == 0,
                "exit_code": process.returncode,
                "stdout": stdout,
                "stderr": stderr,
                "duration_sec": duration,
                "timed_out": False,
            }
        except subprocess.TimeoutExpired:
            process.kill()
            stdout, stderr = process.communicate()
            duration = time.time() - start_time
            return {
                "success": False,
                "exit_code": -1,
                "stdout": stdout,
                "stderr": f"Command timed out after {timeout_seconds} seconds\n{stderr}",
                "duration_sec": duration,
                "timed_out": True,
            }
        except Exception as e:
            duration = time.time() - start_time
            return {
                "success": False,
                "exit_code": -1,
                "stdout": "",
                "stderr": str(e),
                "duration_sec": duration,
                "timed_out": False,
            }
