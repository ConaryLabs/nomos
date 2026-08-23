#!/usr/bin/env python3
"""Remove only documented provider signatures from an otherwise valid Pi stream."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from gate_k_eval_pi_protocol import fail, loads


SIGNATURES = {"textSignature": "text", "thinkingSignature": "thinking"}


def sanitize(value: object, location: str = "event") -> object:
    if type(value) is list:
        return [sanitize(item, f"{location}[{index}]") for index, item in enumerate(value)]
    if type(value) is not dict:
        return value
    result: dict[str, object] = {}
    for key, item in value.items():
        if key in SIGNATURES:
            if value.get("type") != SIGNATURES[key] or type(item) is not str:
                fail(f"{location}.{key} occurs outside its documented content block")
            continue
        result[key] = sanitize(item, f"{location}.{key}")
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("stream", type=Path)
    args = parser.parse_args()
    try:
        lines = args.stream.read_text().splitlines()
        if not lines:
            fail("Pi event stream is empty")
        for index, line in enumerate(lines, 1):
            event = loads(line, f"Pi event line {index}")
            cleaned = sanitize(event, f"event[{index}]")
            print(json.dumps(cleaned, separators=(",", ":"), ensure_ascii=True))
    except (OSError, ValueError) as error:
        print(f"gate-k transcript sanitization: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
