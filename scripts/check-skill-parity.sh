#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

python3 - <<'PY' "$ROOT_DIR"
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1])
agents_dir = root / ".agents" / "skills"
claude_dir = root / ".claude" / "skills"

allowed_one_sided = {"play-persona", "rule-arc"}

agents = {p.name for p in agents_dir.iterdir() if p.is_dir()}
claude = {p.name for p in claude_dir.iterdir() if p.is_dir()}

errors = []

agents_only = sorted(agents - claude - allowed_one_sided)
claude_only = sorted(claude - agents - allowed_one_sided)
for name in agents_only:
    errors.append(f"shared-skill directory exists only under .agents: {name}")
for name in claude_only:
    errors.append(f"shared-skill directory exists only under .claude: {name}")

shared = sorted(agents & claude)

def changed(path: Path, cached: bool) -> bool:
    cmd = ["git", "diff", "--quiet"]
    if cached:
        cmd.append("--cached")
    cmd.extend(["--", str(path.relative_to(root))])
    proc = subprocess.run(cmd, cwd=root)
    return proc.returncode == 1

for name in shared:
    agent_file = agents_dir / name / "SKILL.md"
    claude_file = claude_dir / name / "SKILL.md"
    if not agent_file.exists() or not claude_file.exists():
        errors.append(f"shared skill missing SKILL.md pair: {name}")
        continue

    agent_changed = changed(agent_file, cached=False) or changed(agent_file, cached=True)
    claude_changed = changed(claude_file, cached=False) or changed(claude_file, cached=True)

    if agent_changed != claude_changed:
        side = ".agents" if agent_changed else ".claude"
        errors.append(f"one-sided skill drift for {name}: changed only under {side}")

if errors:
    print(f"skill-parity | errors={len(errors)}")
    for msg in errors:
        print(f"⚠ {msg}")
    raise SystemExit(1)

print(f"skill-parity | ok | shared_checked={len(shared)} allowed_one_sided={len(allowed_one_sided)}")
PY
