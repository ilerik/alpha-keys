#!/usr/bin/env python3
"""Fail if a 20-byte hex address (0x + 40 hex digits) appears in judge-facing files."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATTERN = re.compile(r"0x[0-9a-fA-F]{40}")
ROOTS = [
    "fixtures",
    "gateway/src",
    "web/app",
    "web/lib",
    "attestor",
    "contracts",
    "copier/src",
    "copier/tests",
    "docs/schemas",
]
SKIP = {".venv", "node_modules", "target", "__pycache__"}


def main() -> int:
    hits: list[str] = []
    for folder in ROOTS:
        base = ROOT / folder
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file() or any(part in SKIP for part in path.parts):
                continue
            if path.suffix in {".wasm", ".png", ".lock"}:
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            for index, line in enumerate(text.splitlines(), start=1):
                if PATTERN.search(line):
                    hits.append(f"{path.relative_to(ROOT)}:{index}")
    if hits:
        print("address-shaped hex found:")
        print("\n".join(hits))
        return 1
    print("leak check: no address-shaped hex")
    return 0


if __name__ == "__main__":
    sys.exit(main())
