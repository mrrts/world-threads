#!/usr/bin/env bash
# list_templates.sh — at-a-glance navigation helper for scripts/codex_consult_prompts/
#
# Lists templates by kind (codex-consult-input / Move-2 follow-up / Move-1 future-arc-scaffold)
# with file size + last-modified date so future-session can navigate the pre-flight scaffolding
# directory without grep'ing manually. Reads README.md's three-template-kind taxonomy.
#
# Usage: from project root, run `./scripts/codex_consult_prompts/list_templates.sh`

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "═══════════════════════════════════════════════════════════════"
echo " scripts/codex_consult_prompts/ template inventory"
echo "═══════════════════════════════════════════════════════════════"
echo

print_section () {
  local title="$1"
  local glob="$2"
  echo "── $title ──"
  local found=0
  for f in $glob; do
    if [ -f "$f" ] && [ "$f" != "README.md" ]; then
      local size=$(wc -c < "$f" | tr -d ' ')
      local mtime=$(date -r "$f" +"%Y-%m-%d")
      printf "  %-66s  %5sB  %s\n" "$(basename "$f")" "$size" "$mtime"
      found=$((found + 1))
    fi
  done
  if [ "$found" -eq 0 ]; then
    echo "  (none)"
  fi
  echo
}

print_section "Codex consult inputs (.txt; founding-author hands to external codex)" '*_audit_template.txt'
print_section "Move-2 follow-up templates (.md; pre-authored at \$0; INSERT placeholders fill at codex-verdict-receipt)" '*_followup_*.md'
print_section "Move-1 future-arc scaffolds (.md; pre-authored at \$0 for future-session natural entry)" '*_move1_template.md'

echo "── README.md (directory hygiene + three-template-kinds taxonomy + apparatus discipline) ──"
if [ -f README.md ]; then
  printf "  %-66s  %5sB  %s\n" "README.md" "$(wc -c < README.md | tr -d ' ')" "$(date -r README.md +"%Y-%m-%d")"
fi
echo

echo "═══════════════════════════════════════════════════════════════"
echo " Discipline reminder per README.md:"
echo " - Apparatus does NOT unilaterally fire. Founding-author + codex verdict path."
echo " - Pre-authored templates carry [INSERT] placeholders; do NOT prejudge codex verdict."
echo " - Verbatim codex response inclusion is non-negotiable when filling Move-2."
echo "═══════════════════════════════════════════════════════════════"
