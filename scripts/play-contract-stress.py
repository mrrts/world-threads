#!/usr/bin/env python3
"""Chained stress tests for /play UI contract hooks.

This extends single-turn checks with 3-turn transcript flows, so we can
catch stateful drift patterns (e.g., valid turn -> invalid turn -> recovery).
"""
from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
HOOK_HUD = ["python3", str(ROOT / ".claude/hooks/check-play-hud-present.py")]
HOOK_ASK = ["python3", str(ROOT / ".claude/hooks/check-inline-choosers.py")]
HOOK_CHOOSER = ["python3", str(ROOT / ".claude/hooks/check-play-chooser-format.py")]
HOOK_PLAY_ASK = ["python3", str(ROOT / ".claude/hooks/check-play-askquestion-required.py")]


def run_stop_hook(cmd: list[str], transcript: list[dict]) -> tuple[int, str]:
    with tempfile.NamedTemporaryFile("w", delete=False) as tf:
        for rec in transcript:
            tf.write(json.dumps(rec) + "\n")
        transcript_path = tf.name
    payload = {"transcript_path": transcript_path}
    cp = subprocess.run(cmd, input=json.dumps(payload), text=True, capture_output=True)
    return cp.returncode, cp.stdout.strip()


def run_pretool_hook(cmd: list[str], tool_input: dict) -> tuple[int, str]:
    payload = {"tool_name": "AskUserQuestion", "tool_input": tool_input}
    cp = subprocess.run(cmd, input=json.dumps(payload), text=True, capture_output=True)
    return cp.returncode, cp.stdout.strip()


def is_blocked(stdout: str) -> bool:
    return '"decision": "block"' in stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--smoke",
        action="store_true",
        help="Run only critical contract checks (fast preflight).",
    )
    args = parser.parse_args()

    tests: list[tuple[str, bool]] = []

    # Case 1: Good /play turn at message tail should pass.
    t1 = [
        {"message": {"role": "user", "content": "/play"}},
        {
            "message": {
                "role": "assistant",
                "content": "WORLDTHREADS BUILDER — Turn 106\nBank: $117,650 | Trend: ↑ +$1,400 | 💎 9 | 👑 7\nLast move: Turn 105: Stress suite (+$1,400)",
            }
        },
    ]
    _, out = run_stop_hook(HOOK_HUD, t1)
    tests.append(("hud_tail_good_passes", not is_blocked(out)))

    # Case 2: 3-turn chain, latest assistant /play turn missing HUD should block.
    t2 = [
        {"message": {"role": "user", "content": "/play"}},
        {
            "message": {
                "role": "assistant",
                "content": "WORLDTHREADS BUILDER — Turn 105\nBank: $116,250 | Trend: ↑ +$300 | 💎 9 | 👑 7\nLast move: Turn 104: Contract hardening (+$300)",
            }
        },
        {"message": {"role": "user", "content": "/play"}},
        {"message": {"role": "assistant", "content": "Turn update without header HUD"}},
    ]
    _, out = run_stop_hook(HOOK_HUD, t2)
    tests.append(("hud_chain_latest_missing_blocks", is_blocked(out)))

    # Case 3: AskUserQuestion chain recovery: latest assistant includes tool_use, pass.
    t3 = [
        {"message": {"role": "user", "content": "next"}},
        {"message": {"role": "assistant", "content": [{"type": "text", "text": "no chooser"}]}},
        {"message": {"role": "user", "content": "retry"}},
        {
            "message": {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "fixed"},
                    {"type": "tool_use", "name": "AskUserQuestion", "input": {"questions": []}},
                ],
            }
        },
    ]
    _, out = run_stop_hook(HOOK_ASK, t3)
    tests.append(("ask_chain_recovery_passes", not is_blocked(out)))

    # Case 4: AskUserQuestion chain latest missing should block.
    t4 = [
        {"message": {"role": "user", "content": "next"}},
        {
            "message": {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "ok"},
                    {"type": "tool_use", "name": "AskUserQuestion", "input": {"questions": []}},
                ],
            }
        },
        {"message": {"role": "user", "content": "again"}},
        {"message": {"role": "assistant", "content": [{"type": "text", "text": "forgot chooser"}]}},
    ]
    _, out = run_stop_hook(HOOK_ASK, t4)
    tests.append(("ask_chain_latest_missing_blocks", is_blocked(out)))

    # Case 4b: Real regression shape from play flow:
    # user is in /play and assistant closes with plain text (no AskUserQuestion).
    t4b = [
        {"message": {"role": "user", "content": "/play"}},
        {
            "message": {
                "role": "assistant",
                "content": "WORLDTHREADS BUILDER — Turn 111\nBank: $123,850 | Trend: ↑ +$1,100 | 💎 9 | 👑 7\nLast move: Turn 110: receipt (+$1,100)\n[A] ... [B] ... [C] ... [D] ...",
            }
        },
    ]
    _, out = run_stop_hook(HOOK_ASK, t4b)
    tests.append(("ask_play_plain_text_close_blocks", is_blocked(out)))

    # Case 5: PreTool chooser-format pass with 4 options and bounty labels.
    q_pass = {
        "questions": [
            {
                "question": "Turn 108 move",
                "options": [
                    {"label": "[A] (+$1,200) Option A"},
                    {"label": "[B] (+$1,100) Option B"},
                    {"label": "[C] (+$800) Option C"},
                    {"label": "[D] (+$300) Provide your own next move"},
                ],
            }
        ]
    }
    _, out = run_pretool_hook(HOOK_CHOOSER, q_pass)
    tests.append(("pretool_chooser_format_good_passes", not is_blocked(out)))

    # Case 6: PreTool chooser-format blocks 3-option cardinality.
    q_bad_count = {
        "questions": [
            {
                "question": "Turn 108 move",
                "options": [
                    {"label": "[A] (+$1,200) Option A"},
                    {"label": "[B] (+$1,100) Option B"},
                    {"label": "[C] (+$800) Option C"},
                ],
            }
        ]
    }
    _, out = run_pretool_hook(HOOK_CHOOSER, q_bad_count)
    tests.append(("pretool_chooser_cardinality_blocks", is_blocked(out)))

    # Case 7: PreTool chooser-format blocks missing bounty label on non-user slot.
    q_bad_bounty = {
        "questions": [
            {
                "question": "Turn 108 move",
                "options": [
                    {"label": "[A] Option A"},
                    {"label": "[B] (+$1,100) Option B"},
                    {"label": "[C] (+$800) Option C"},
                    {"label": "[D] (+$300) Provide your own next move"},
                ],
            }
        ]
    }
    _, out = run_pretool_hook(HOOK_CHOOSER, q_bad_bounty)
    tests.append(("pretool_chooser_missing_bounty_blocks", is_blocked(out)))

    # Case 8: /play-specific hook blocks chooser-answer pause turn with no work.
    t8 = [
        {
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "content": "User has answered with option A",
                    }
                ],
            }
        },
        {
            "message": {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "WORLDTHREADS BUILDER — Turn 124\nSelected A. Executing now."},
                    {"type": "tool_use", "name": "AskUserQuestion", "input": {"questions": []}},
                ],
            }
        },
    ]
    _, out = run_stop_hook(HOOK_PLAY_ASK, t8)
    tests.append(("play_hook_blocks_confirmation_only_pause", is_blocked(out)))

    # Case 9: /play-specific hook passes when real work tool call happens.
    t9 = [
        {
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "content": "User has answered with option A",
                    }
                ],
            }
        },
        {
            "message": {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "WORLDTHREADS BUILDER — Turn 124"},
                    {"type": "tool_use", "name": "Shell", "input": {"command": "echo run"}},
                ],
            }
        },
        {
            "message": {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "Work done."},
                    {"type": "tool_use", "name": "AskUserQuestion", "input": {"questions": []}},
                ],
            }
        },
    ]
    _, out = run_stop_hook(HOOK_PLAY_ASK, t9)
    tests.append(("play_hook_allows_after_work_tool_use", not is_blocked(out)))

    if args.smoke:
        smoke_names = {
            "hud_tail_good_passes",
            "hud_chain_latest_missing_blocks",
            "ask_play_plain_text_close_blocks",
            "pretool_chooser_format_good_passes",
            "play_hook_blocks_confirmation_only_pause",
        }
        tests = [t for t in tests if t[0] in smoke_names]

    label = "play-contract-smoke" if args.smoke else "play-contract-stress"
    passed = sum(1 for _, ok in tests if ok)
    print(f"{label}: passed {passed}/{len(tests)}")
    for name, ok in tests:
        print(f"- {'PASS' if ok else 'FAIL'}: {name}")

    return 0 if passed == len(tests) else 1


if __name__ == "__main__":
    raise SystemExit(main())
