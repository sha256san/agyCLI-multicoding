"""Configuration file reader and parser for Multi-Agent Development Orchestrator."""

import json
import os
from typing import Any, Dict


def parse_simple_yaml(text: str) -> Dict[str, Any]:
    """Lightweight simple YAML parser supporting key-values, lists, and basic nested dicts

    for environments without PyYAML.
    """
    result: Dict[str, Any] = {}
    current_key = None
    lines = text.splitlines()

    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue

        if line.startswith("  - ") and current_key:
            item = stripped[2:].strip().strip('"\'')
            if current_key not in result or not isinstance(result[current_key], list):
                result[current_key] = []
            result[current_key].append(item)
            continue

        if ":" in stripped:
            parts = stripped.split(":", 1)
            key = parts[0].strip()
            val = parts[1].strip()

            if val == "":
                current_key = key
                result[key] = []
            else:
                val = val.strip('"\'')
                if val.lower() == "true":
                    val_parsed: Any = True
                elif val.lower() == "false":
                    val_parsed = False
                elif val.isdigit():
                    val_parsed = int(val)
                else:
                    val_parsed = val
                result[key] = val_parsed
                current_key = None

    return result


def load_config_file(filepath: str) -> Dict[str, Any]:
    """Load configuration from YAML or JSON file."""
    if not os.path.exists(filepath):
        return {}

    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    if filepath.endswith(".json"):
        return json.loads(content)

    # Try PyYAML if available
    try:
        import yaml  # type: ignore
        return yaml.safe_load(content) or {}
    except ImportError:
        return parse_simple_yaml(content)
