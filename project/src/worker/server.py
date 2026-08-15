"""Worker Agent HTTP REST API server."""

import argparse
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import socketserver
import sys
import threading
import time
from typing import Optional
from urllib.parse import urlparse

from project.src.common.constants import DEFAULT_WORKER_PORTS
from project.src.common.schemas import (
    AgentExecutionStatus,
    AgentRole,
    AgentStatus,
    TaskRequest,
    TaskResult,
)
from project.src.worker.agent_logic import create_agent_handler


class WorkerState:
    def __init__(self, agent_id: str, role: AgentRole, host: str, port: int):
        self.agent_id = agent_id
        self.role = role
        self.host = host
        self.port = port
        self.status = AgentExecutionStatus.IDLE
        self.current_task: Optional[TaskRequest] = None
        self.latest_result: Optional[TaskResult] = None
        self.started_at: Optional[str] = None
        self.progress_percent: int = 0
        self.lock = threading.Lock()
        self.current_thread: Optional[threading.Thread] = None
        self.cancel_requested = False
        self.handler = create_agent_handler(agent_id, role)

    def get_agent_status(self) -> AgentStatus:
        with self.lock:
            return AgentStatus(
                agent_id=self.agent_id,
                role=self.role,
                status=self.status,
                current_task_id=self.current_task.task_id if self.current_task else None,
                started_at=self.started_at,
                progress_percent=self.progress_percent,
                host=self.host,
                port=self.port,
            )

    def start_task(self, task: TaskRequest) -> bool:
        with self.lock:
            if self.status == AgentExecutionStatus.RUNNING:
                return False
            self.status = AgentExecutionStatus.RUNNING
            self.current_task = task
            self.started_at = datetime.now(timezone.utc).isoformat()
            self.progress_percent = 10
            self.cancel_requested = False

        def _worker_run():
            try:
                result = self.handler.execute_task(task)
                with self.lock:
                    self.latest_result = result
                    self.status = AgentExecutionStatus.IDLE
                    self.current_task = None
                    self.progress_percent = 100
            except Exception as e:
                with self.lock:
                    self.latest_result = TaskResult(
                        task_id=task.task_id,
                        agent_id=self.agent_id,
                        status="FAILED",
                        summary=f"Unhandled exception during task execution: {str(e)}",
                        errors=[str(e)],
                    )
                    self.status = AgentExecutionStatus.ERROR
                    self.current_task = None
                    self.progress_percent = 0

        t = threading.Thread(target=_worker_run, daemon=True)
        self.current_thread = t
        t.start()
        return True

    def cancel_task(self) -> bool:
        with self.lock:
            if self.status != AgentExecutionStatus.RUNNING:
                return False
            self.cancel_requested = True
            self.status = AgentExecutionStatus.IDLE
            self.current_task = None
            return True


class WorkerRequestHandler(BaseHTTPRequestHandler):
    worker_state: WorkerState  # injected by server

    def _send_json_response(self, status_code: int, data: dict):
        response_body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status_code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(response_body)))
        self.end_headers()
        self.wfile.write(response_body)

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path

        if path == "/health":
            self._send_json_response(200, {
                "status": "healthy",
                "agent_id": self.worker_state.agent_id,
                "role": self.worker_state.role.value,
                "timestamp": datetime.now(timezone.utc).isoformat(),
            })
        elif path == "/status":
            agent_status = self.worker_state.get_agent_status()
            self._send_json_response(200, agent_status.to_dict())
        elif path == "/result":
            with self.worker_state.lock:
                if self.worker_state.latest_result:
                    self._send_json_response(200, self.worker_state.latest_result.to_dict())
                else:
                    self._send_json_response(404, {"error": "No result available"})
        else:
            self._send_json_response(404, {"error": "Endpoint not found"})

    def do_POST(self):
        parsed = urlparse(self.path)
        path = parsed.path

        content_len = int(self.headers.get("Content-Length", 0))
        post_body = self.rfile.read(content_len).decode("utf-8") if content_len > 0 else "{}"

        if path == "/task":
            try:
                data = json.loads(post_body)
                task_req = TaskRequest.from_dict(data)
                accepted = self.worker_state.start_task(task_req)
                if accepted:
                    self._send_json_response(202, {
                        "message": "Task accepted and started",
                        "task_id": task_req.task_id,
                        "agent_id": self.worker_state.agent_id,
                    })
                else:
                    self._send_json_response(409, {
                        "error": "Agent is currently busy with another task",
                        "agent_id": self.worker_state.agent_id,
                    })
            except Exception as e:
                self._send_json_response(400, {"error": f"Invalid task payload: {str(e)}"})

        elif path == "/cancel":
            cancelled = self.worker_state.cancel_task()
            if cancelled:
                self._send_json_response(200, {"message": "Task cancelled successfully"})
            else:
                self._send_json_response(400, {"error": "No running task to cancel"})
        else:
            self._send_json_response(404, {"error": "Endpoint not found"})

    def log_message(self, format, *args):
        # Quiet standard output logs in normal operations
        pass


class ThreadedHTTPServer(socketserver.ThreadingMixIn, HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


def run_worker_server(agent_id: str, role: AgentRole, host: str = "0.0.0.0", port: Optional[int] = None):
    if port is None:
        port = DEFAULT_WORKER_PORTS.get(agent_id, 8001)

    state = WorkerState(agent_id=agent_id, role=role, host=host, port=port)

    class CustomHandler(WorkerRequestHandler):
        worker_state = state

    server = ThreadedHTTPServer((host, port), CustomHandler)
    print(f"[*] Worker Agent [{agent_id}] ({role.value}) listening on http://{host}:{port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print(f"\n[!] Stopping Worker Agent [{agent_id}]")
    finally:
        server.server_close()


def main():
    parser = argparse.ArgumentParser(description="Multi-Agent Worker REST API Server")
    parser.add_argument("--agent-id", required=True, help="Agent unique ID (e.g. agent-a)")
    parser.add_argument("--role", required=True, choices=[r.value for r in AgentRole], help="Agent role")
    parser.add_argument("--host", default="0.0.0.0", help="Host address to bind")
    parser.add_argument("--port", type=int, default=None, help="Port to listen on")

    args = parser.parse_args()
    role_enum = AgentRole(args.role)
    run_worker_server(agent_id=args.agent_id, role=role_enum, host=args.host, port=args.port)


if __name__ == "__main__":
    main()
