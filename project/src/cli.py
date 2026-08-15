"""Command Line Interface for Multi-Agent Development Orchestrator (`mag`)."""

import argparse
import os
import subprocess
import sys
import threading
import time
from typing import List, Optional

from project.src.common.constants import DEFAULT_WORKER_PORTS
from project.src.common.schemas import AgentRole, TaskStatus
from project.src.manager.diagnostics import EnvDoctor
from project.src.manager.git_manager import GitManager
from project.src.manager.orchestrator import Orchestrator
from project.src.worker.server import run_worker_server


def print_banner():
    banner = r"""
  __  __         _ _   _              _                    _   
 |  \/  |_  _ __| | |_(_)___ __ _ _ _ (_)___  _ __  __ _ __| |  
 | |\/| | || / _| |  _| / _ \ _` | '_/| (_-< | '  \/ _` / _` |_ 
 |_|  |_|\_,_\__|_|\__|_\___/\__,_|_|  |_/__/ |_|_|_\__,_\__, (_)
                                                          |__/  
 Multi-Agent Software Development Orchestrator (`mag`)
    """
    print(banner)


def cmd_init(args):
    project_name = args.name or "my-agent-project"
    print(f"[*] Initializing Multi-Agent project '{project_name}'...")

    os.makedirs("src", exist_ok=True)
    os.makedirs("tests", exist_ok=True)
    os.makedirs("docs", exist_ok=True)
    os.makedirs("logs", exist_ok=True)

    git = GitManager()
    git.init_repo()

    print(f"[✓] Project '{project_name}' successfully initialized!")
    print("    - Repository: Initialized Git")
    print("    - Directories: src/, tests/, docs/, logs/")
    print("    - Configuration: project/project.yaml")


def cmd_status(args):
    print("=" * 70)
    print(" MULTI-AGENT ORCHESTRATOR STATUS")
    print("=" * 70)

    orch = Orchestrator()
    print(f" Manager Node:   RUNNING  (SQLite: {orch.db.db_path})")
    print("-" * 70)
    print(f"{'AGENT ID':<12} {'ROLE':<14} {'PORT':<8} {'HEALTH':<12} {'CURRENT TASK'}")
    print("-" * 70)

    agents = [
        ("agent-a", "Developer", 8001),
        ("agent-b", "Tester", 8002),
        ("agent-c", "Reviewer", 8003),
        ("agent-d", "Security", 8004),
        ("agent-e", "Researcher", 8005),
    ]

    for agent_id, role, port in agents:
        is_healthy = orch.worker_client.check_health("127.0.0.1", port)
        status_info = orch.worker_client.get_status("127.0.0.1", port)
        current_task = status_info.current_task_id if (status_info and status_info.current_task_id) else "-"
        health_str = "[ONLINE]" if is_healthy else "[OFFLINE]"
        print(f"{agent_id:<12} {role:<14} {port:<8} {health_str:<12} {current_task}")

    print("-" * 70)
    print("\nRecent Tasks:")
    tasks = orch.task_manager.list_tasks()
    if not tasks:
        print("  (No tasks found in database)")
    else:
        print(f"  {'TASK ID':<10} {'STATUS':<15} {'AGENT':<10} {'TITLE'}")
        for t in tasks[-10:]:
            status_val = t.status.value if hasattr(t.status, "value") else str(t.status)
            print(f"  {t.task_id:<10} {status_val:<15} {t.assigned_agent:<10} {t.title[:35]}")
    print("=" * 70)


def start_local_worker_threads() -> List[threading.Thread]:
    """Start in-process worker HTTP servers in daemon threads for quick local testing."""
    agents = [
        ("agent-a", AgentRole.DEVELOPER, 8001),
        ("agent-b", AgentRole.TESTER, 8002),
        ("agent-c", AgentRole.REVIEWER, 8003),
        ("agent-d", AgentRole.SECURITY, 8004),
        ("agent-e", AgentRole.RESEARCHER, 8005),
    ]

    threads = []
    for agent_id, role, port in agents:
        t = threading.Thread(
            target=run_worker_server,
            kwargs={"agent_id": agent_id, "role": role, "host": "127.0.0.1", "port": port},
            daemon=True,
        )
        t.start()
        threads.append(t)

    # Wait briefly for servers to bind
    time.sleep(0.3)
    return threads


def cmd_run(args):
    prompt = args.prompt
    print_banner()
    print(f"[*] Received user instruction: \"{prompt}\"\n")

    orch = Orchestrator()

    # Check if workers are running; if offline, auto-spawn background local workers
    health = orch.check_all_agents_health()
    offline_agents = [a for a, online in health.items() if not online]
    if offline_agents:
        print(f"[*] Starting local Worker agents for: {', '.join(offline_agents)}...")
        start_local_worker_threads()

    print("[*] Manager: Analyzing requirement and decomposing into task DAG...")
    tasks = orch.decompose_requirement(prompt)
    print(f"[✓] Generated {len(tasks)} tasks:")
    for t in tasks:
        deps = f"(depends on: {', '.join(t.dependencies)})" if t.dependencies else "(root)"
        print(f"    - [{t.task_id}] {t.role.value.upper():<12} -> Assigned to: {t.assigned_agent} {deps}")

    print("\n[*] Starting Autonomous Multi-Agent Orchestration Loop...\n")
    success = orch.run_orchestration_loop()

    print("\n" + "=" * 70)
    if success:
        print(" [✓] MULTI-AGENT DEVELOPMENT WORKFLOW COMPLETED SUCCESSFULLY!")
        print("=" * 70)
        print("  - All implementation, testing, review, and security stages passed.")
        print("  - Changes are validated and tracked in SQLite database.")
    else:
        print(" [!] MULTI-AGENT WORKFLOW FINISHED WITH SOME FAILED TASKS")
        print("=" * 70)

    # Print summary of tasks
    for t in orch.task_manager.list_tasks()[-len(tasks):]:
        res_summary = t.result.summary if t.result else "No result"
        status_val = t.status.value if hasattr(t.status, "value") else str(t.status)
        print(f"  • [{t.task_id}] {status_val:<12} | {res_summary}")
    print("=" * 70)


def cmd_task(args):
    orch = Orchestrator()
    if args.task_action == "list":
        tasks = orch.task_manager.list_tasks()
        print(f"{'TASK ID':<10} {'STATUS':<15} {'AGENT':<10} {'TITLE'}")
        print("-" * 65)
        for t in tasks:
            status_val = t.status.value if hasattr(t.status, "value") else str(t.status)
            print(f"{t.task_id:<10} {status_val:<15} {t.assigned_agent:<10} {t.title}")
    elif args.task_action == "show":
        task = orch.task_manager.get_task(args.task_id)
        if not task:
            print(f"Task '{args.task_id}' not found.")
            return
        print(f"Task ID:     {task.task_id}")
        print(f"Title:       {task.title}")
        print(f"Description: {task.description}")
        print(f"Agent:       {task.assigned_agent} ({task.role.value})")
        print(f"Status:      {task.status.value}")
        print(f"Retries:     {task.retry_count}/{task.max_retries}")
        if task.result:
            print(f"Result Status:  {task.result.status}")
            print(f"Result Summary: {task.result.summary}")
            if task.result.files_changed:
                print(f"Files Changed:  {', '.join(task.result.files_changed)}")


def cmd_logs(args):
    orch = Orchestrator()
    logs = orch.db.get_logs(agent_id=args.agent_id, limit=args.limit or 50)
    print(f"--- Showing last {len(logs)} logs ---")
    for log in reversed(logs):
        print(f"[{log['timestamp']}] [{log['level']}] (Agent: {log['agent_id'] or 'System'}) {log['message']}")


def cmd_doctor(args):
    print("[*] Running EnvDoctor system diagnostics...")
    report = EnvDoctor.diagnose_system()
    print(f"  Python Version: {report['python_version'].split()[0]}")
    print(f"  OS Name:        {report['os_name']}")
    print("\n  Tool Availability:")
    for tool, info in report["tools"].items():
        status = f"[✓] Found at {info['path']}" if info["installed"] else "[✗] Missing"
        print(f"    - {tool:<10}: {status}")

    if report["is_healthy"]:
        print("\n[✓] Environment check passed! All core development dependencies are ready.")
    else:
        print(f"\n[!] Warnings/Issues detected: {', '.join(report['issues'])}")


def main():
    parser = argparse.ArgumentParser(
        description="Multi-Agent Development Orchestrator (`mag`) CLI",
        usage="mag <command> [<args>] | mag \"<natural language instruction>\"",
    )

    subparsers = parser.add_subparsers(dest="command")

    # init
    init_parser = subparsers.add_parser("init", help="Initialize new project")
    init_parser.add_argument("name", nargs="?", default="my-project", help="Project name")

    # status
    subparsers.add_parser("status", help="Show orchestrator and agent status")

    # run
    run_parser = subparsers.add_parser("run", help="Run autonomous multi-agent task workflow")
    run_parser.add_argument("prompt", help="Instruction or requirement prompt")

    # task
    task_parser = subparsers.add_parser("task", help="Manage tasks")
    task_sub = task_parser.add_subparsers(dest="task_action")
    task_sub.add_parser("list", help="List all tasks")
    show_p = task_sub.add_parser("show", help="Show task details")
    show_p.add_argument("task_id", help="Task ID to inspect")

    # logs
    log_parser = subparsers.add_parser("logs", help="View orchestrator logs")
    log_parser.add_argument("agent_id", nargs="?", default=None, help="Filter by Agent ID")
    log_parser.add_argument("--limit", type=int, default=50, help="Number of logs to show")

    # doctor
    subparsers.add_parser("doctor", help="Run environment diagnostic checks (envdoctor)")

    # If first argument isn't a known subcommand and doesn't start with '-', treat it as a natural language prompt
    if len(sys.argv) > 1 and sys.argv[1] not in subparsers.choices and not sys.argv[1].startswith("-"):
        prompt_text = " ".join(sys.argv[1:])
        class Args:
            prompt = prompt_text
        cmd_run(Args())
        return

    args = parser.parse_args()

    if args.command == "init":
        cmd_init(args)
    elif args.command == "status":
        cmd_status(args)
    elif args.command == "run":
        cmd_run(args)
    elif args.command == "task":
        cmd_task(args)
    elif args.command == "logs":
        cmd_logs(args)
    elif args.command == "doctor":
        cmd_doctor(args)
    else:
        print_banner()
        parser.print_help()


if __name__ == "__main__":
    main()
