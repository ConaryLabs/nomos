#!/usr/bin/env python3
"""Parse one strict JSON document with the shared Gate K loader."""

import sys
from pathlib import Path

from gate_k_eval_pi_protocol import loads


try:
    if len(sys.argv) != 2:
        raise ValueError("usage: gate-k-eval-validate-json.py FILE")
    source = sys.stdin.read() if sys.argv[1] == "-" else Path(sys.argv[1]).read_text()
    value = loads(source, "JSON document")
    if type(value) not in (dict, list):
        raise ValueError("top-level JSON value must be an object or array")
except (OSError, ValueError) as error:
    print(f"gate-k JSON validation: FAIL: {error}", file=sys.stderr)
    raise SystemExit(1)
