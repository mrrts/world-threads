#!/usr/bin/env bash
# One-time setup: store an OpenAI API key in the macOS Keychain so
# Claude Code sessions can reuse it for analysis runs (eval harness,
# ad-hoc scripts) without re-pasting the key each session.
#
# Security model:
# - The key is stored in Keychain, encrypted at rest, unlocked with
#   your login password. Not on disk in plaintext, not in git.
# - Claude Code sessions that need the key will read it via the
#   `security` CLI (see get-claude-code-llm-key.sh). This means any
#   script I run on your behalf CAN read the key — it has to, to
#   call the API. "Secure" here means "secure at rest and across
#   sessions," not "hidden from the agent."
# - The key lives under service="claude-code-llm-api-key",
#   account="$USER". To remove it later:
#     security delete-generic-password -s claude-code-llm-api-key
#
# Run this once:   bash scripts/setup-claude-code-llm-key.sh
# Retrieve later:  bash scripts/get-claude-code-llm-key.sh

set -euo pipefail

SERVICE="claude-code-llm-api-key"
ACCOUNT="${USER}"

if ! command -v security >/dev/null 2>&1; then
    echo "This script uses macOS 'security'. Only runs on macOS." >&2
    exit 1
fi

# Hidden input so the key isn't visible on screen or in shell history.
printf "Paste your OpenAI API key (input is hidden): "
stty -echo
IFS= read -r KEY
stty echo
echo

KEY="${KEY#"${KEY%%[![:space:]]*}"}"   # trim leading whitespace
KEY="${KEY%"${KEY##*[![:space:]]}"}"   # trim trailing whitespace

if [ -z "${KEY}" ]; then
    echo "No key given. Nothing stored." >&2
    exit 1
fi

# -U updates if it exists, otherwise creates.
# -w <pw> sets the password without prompting again.
security add-generic-password -U -s "${SERVICE}" -a "${ACCOUNT}" -w "${KEY}"

echo "Stored in Keychain (service: ${SERVICE}, account: ${ACCOUNT})."
echo
echo "To verify:  bash scripts/get-claude-code-llm-key.sh | head -c 10; echo ..."
echo "To delete:  security delete-generic-password -s ${SERVICE}"
