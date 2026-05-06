#!/usr/bin/env python3
"""Compatibility wrapper for the Empiricon sacred-payload audit."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from sacred_payload_audit import main as sacred_payload_main  # noqa: E402


if __name__ == "__main__":
    sys.argv.insert(1, "--profile")
    sys.argv.insert(2, "empiricon")
    sacred_payload_main()
