#!/usr/bin/env python3
"""Parse one JSON document while rejecting duplicate object keys."""

import json
import sys
from pathlib import Path


def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def reject_constant(value: str) -> None:
    raise ValueError(f"non-finite JSON number: {value}")


try:
    if len(sys.argv) != 2:
        raise ValueError("usage: gate-k-eval-validate-json.py FILE")
    source = sys.stdin.read() if sys.argv[1] == "-" else Path(sys.argv[1]).read_text()
    value = json.loads(
        source,
        object_pairs_hook=reject_duplicates,
        parse_constant=reject_constant,
    )
    if type(value) not in (dict, list):
        raise ValueError("top-level JSON value must be an object or array")
except (OSError, json.JSONDecodeError, ValueError) as error:
    print(f"gate-k JSON validation: FAIL: {error}", file=sys.stderr)
    raise SystemExit(1)
