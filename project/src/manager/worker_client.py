"""HTTP REST Client for communicating with Worker Agent containers."""

import json
import urllib.error
import urllib.request
from typing import Any, Dict, Optional

from project.src.common.schemas import AgentStatus, TaskRequest, TaskResult


class WorkerClient:
    def __init__(self, default_timeout: float = 10.0):
        self.default_timeout = default_timeout

    def _make_request(
        self,
        url: str,
        method: str = "GET",
        data: Optional[Dict[str, Any]] = None,
        timeout: Optional[float] = None,
    ) -> Dict[str, Any]:
        req_timeout = timeout or self.default_timeout
        headers = {"Content-Type": "application/json; charset=utf-8"}
        body_bytes = json.dumps(data).encode("utf-8") if data is not None else None

        req = urllib.request.Request(url, data=body_bytes, headers=headers, method=method)

        try:
            with urllib.request.urlopen(req, timeout=req_timeout) as resp:
                resp_bytes = resp.read()
                if resp_bytes:
                    return json.loads(resp_bytes.decode("utf-8"))
                return {}
        except urllib.error.HTTPError as e:
            err_body = e.read().decode("utf-8", errors="ignore")
            try:
                err_json = json.loads(err_body)
                return {"error": err_json.get("error", str(e)), "status_code": e.code}
            except Exception:
                return {"error": f"HTTP {e.code}: {e.reason}", "status_code": e.code}
        except Exception as e:
            return {"error": str(e), "status_code": 0}

    def check_health(self, host: str, port: int) -> bool:
        url = f"http://{host}:{port}/health"
        res = self._make_request(url, method="GET", timeout=2.0)
        return res.get("status") == "healthy"

    def send_task(self, host: str, port: int, task: TaskRequest) -> Dict[str, Any]:
        url = f"http://{host}:{port}/task"
        return self._make_request(url, method="POST", data=task.to_dict(), timeout=5.0)

    def get_status(self, host: str, port: int) -> Optional[AgentStatus]:
        url = f"http://{host}:{port}/status"
        res = self._make_request(url, method="GET", timeout=3.0)
        if "error" in res:
            return None
        return AgentStatus.from_dict(res)

    def get_result(self, host: str, port: int) -> Optional[TaskResult]:
        url = f"http://{host}:{port}/result"
        res = self._make_request(url, method="GET", timeout=5.0)
        if "error" in res:
            return None
        return TaskResult.from_dict(res)

    def cancel_task(self, host: str, port: int) -> bool:
        url = f"http://{host}:{port}/cancel"
        res = self._make_request(url, method="POST", timeout=3.0)
        return "error" not in res
