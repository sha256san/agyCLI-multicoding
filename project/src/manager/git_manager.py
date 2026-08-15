"""Git version control and worktree management for Multi-Agent Orchestrator."""

import os
import subprocess
from typing import Dict, List, Optional, Tuple


class GitManager:
    def __init__(self, repo_path: str = "."):
        self.repo_path = os.path.abspath(repo_path)

    def _run_git(self, args: List[str], cwd: Optional[str] = None) -> Tuple[bool, str, str]:
        cmd = ["git"] + args
        work_dir = cwd or self.repo_path
        try:
            p = subprocess.run(
                cmd,
                cwd=work_dir,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )
            return p.returncode == 0, p.stdout.strip(), p.stderr.strip()
        except Exception as e:
            return False, "", str(e)

    def is_git_repo(self) -> bool:
        ok, _, _ = self._run_git(["rev-parse", "--is-inside-work-tree"])
        return ok

    def init_repo(self) -> bool:
        if not self.is_git_repo():
            ok, _, _ = self._run_git(["init"])
            if ok:
                # Create initial commit if empty
                self._run_git(["config", "user.name", "Multi-Agent Manager"])
                self._run_git(["config", "user.email", "manager@antigravity.local"])
                with open(os.path.join(self.repo_path, ".gitignore"), "a") as f:
                    f.write("\nlogs/\n__pycache__/\n*.pyc\n.pytest_cache/\n")
                self._run_git(["add", "."])
                self._run_git(["commit", "-m", "chore: initial project repository"])
            return ok
        return True

    def create_branch(self, branch_name: str, base_branch: str = "main") -> bool:
        ok, _, _ = self._run_git(["checkout", "-B", branch_name])
        return ok

    def create_worktree(self, branch_name: str, worktree_path: str) -> bool:
        """Create a separate git worktree directory for an agent."""
        os.makedirs(os.path.dirname(os.path.abspath(worktree_path)), exist_ok=True)
        ok, out, err = self._run_git(["worktree", "add", "-B", branch_name, worktree_path, "main"])
        return ok

    def remove_worktree(self, worktree_path: str) -> bool:
        ok, _, _ = self._run_git(["worktree", "remove", "--force", worktree_path])
        return ok

    def get_current_commit(self, cwd: Optional[str] = None) -> Optional[str]:
        ok, out, _ = self._run_git(["rev-parse", "--short", "HEAD"], cwd=cwd)
        return out if ok else None

    def get_diff(self, base_branch: str = "main", branch_name: Optional[str] = None) -> str:
        target = branch_name or "HEAD"
        ok, out, _ = self._run_git(["diff", f"{base_branch}...{target}"])
        return out if ok else ""

    def merge_branch(self, branch_name: str, target_branch: str = "main", squash: bool = False) -> Tuple[bool, str]:
        # Switch to target branch
        ok, _, err = self._run_git(["checkout", target_branch])
        if not ok:
            return False, f"Failed to checkout {target_branch}: {err}"

        # Merge
        merge_args = ["merge", branch_name]
        if squash:
            merge_args.insert(1, "--squash")

        ok, out, err = self._run_git(merge_args)
        if ok:
            if squash:
                self._run_git(["commit", "-m", f"feat: merge {branch_name}"])
            return True, f"Successfully merged {branch_name} into {target_branch}"
        else:
            self._run_git(["merge", "--abort"])
            return False, f"Merge conflict or failure: {err}"
