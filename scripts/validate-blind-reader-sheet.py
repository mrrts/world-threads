#!/usr/bin/env python3
"""Validate strict blind-reader score sheet CSV integrity.

Usage:
  python3 scripts/validate-blind-reader-sheet.py
  python3 scripts/validate-blind-reader-sheet.py --csv reports/<file>.csv
"""

from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CSV = ROOT / "reports/2026-04-30-0115-blind-reader-score-sheet-v1.csv"

REQUIRED_COLUMNS = [
    "run_id",
    "reader_id",
    "reader_bucket",
    "excluded",
    "exclusion_reason",
    "packet_version",
    "packet_hash",
    "randomized_order",
    "cell_id",
    "authenticity_1_5",
    "doctrinal_weight_1_5",
    "tradition_recognition_1_5",
    "reader_notes",
]

RATING_COLUMNS = [
    "authenticity_1_5",
    "doctrinal_weight_1_5",
    "tradition_recognition_1_5",
]

ALLOWED_EXCLUDED = {"true", "false"}


def _parse_bool(raw: str) -> bool:
    v = raw.strip().lower()
    if v not in ALLOWED_EXCLUDED:
        raise ValueError(f"excluded must be one of {sorted(ALLOWED_EXCLUDED)}, got {raw!r}")
    return v == "true"


def _validate_rating(raw: str, row_num: int, col: str) -> None:
    val = raw.strip()
    if not val:
        raise ValueError(f"row {row_num}: {col} is required for included rows")
    try:
        num = int(val)
    except ValueError as exc:
        raise ValueError(f"row {row_num}: {col} must be integer 1-5, got {raw!r}") from exc
    if num < 1 or num > 5:
        raise ValueError(f"row {row_num}: {col} must be in [1,5], got {num}")


def validate(csv_path: Path, allow_unscored: bool = False) -> None:
    if not csv_path.exists():
        raise SystemExit(f"missing csv: {csv_path}")

    with csv_path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        fieldnames = reader.fieldnames or []

        missing = [c for c in REQUIRED_COLUMNS if c not in fieldnames]
        if missing:
            raise SystemExit(f"missing required columns: {', '.join(missing)}")

        for i, row in enumerate(reader, start=2):
            try:
                excluded = _parse_bool(row["excluded"])
            except ValueError as e:
                raise SystemExit(f"row {i}: {e}") from e

            reason = row["exclusion_reason"].strip()
            if excluded and not reason:
                raise SystemExit(
                    f"row {i}: exclusion_reason required when excluded=true"
                )
            if (not excluded) and reason:
                raise SystemExit(
                    f"row {i}: exclusion_reason must be empty when excluded=false"
                )

            if excluded:
                # Excluded rows should not carry scored ratings.
                bad = [c for c in RATING_COLUMNS if row[c].strip()]
                if bad:
                    raise SystemExit(
                        f"row {i}: excluded row must leave rating columns blank: {', '.join(bad)}"
                    )
            else:
                for col in RATING_COLUMNS:
                    if allow_unscored and not row[col].strip():
                        continue
                    try:
                        _validate_rating(row[col], i, col)
                    except ValueError as e:
                        raise SystemExit(str(e)) from e

            # Basic required non-empty identity fields.
            for col in ("run_id", "reader_id", "reader_bucket", "packet_version", "cell_id"):
                if not row[col].strip():
                    raise SystemExit(f"row {i}: {col} is required and must be non-empty")

    rel = csv_path.relative_to(ROOT) if csv_path.is_relative_to(ROOT) else csv_path
    print(f"blind-reader sheet ok: {rel}")


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--csv", type=Path, default=DEFAULT_CSV)
    p.add_argument(
        "--allow-unscored",
        action="store_true",
        help="allow blank rating cells for template/pre-collection sheets",
    )
    args = p.parse_args()
    csv_path = args.csv if args.csv.is_absolute() else ROOT / args.csv
    validate(csv_path.resolve(), allow_unscored=args.allow_unscored)


if __name__ == "__main__":
    main()
