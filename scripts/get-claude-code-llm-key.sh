#!/usr/bin/env bash
# Print the user's stored LLM API key to stdout. Silent on failure
# (prints nothing, exits 0) so callers can `key=$(... || echo "")`
# without bash error-trapping.
#
# Storage: macOS Keychain under service="claude-code-llm-api-key".
# Set it up with: bash scripts/setup-claude-code-llm-key.sh
#
# Any script that needs an LLM for Claude Code analysis work should
# prefer `LLM_API_KEY` env var first, then fall back to this helper.

SERVICE="claude-code-llm-api-key"
security find-generic-password -s "${SERVICE}" -w 2>/dev/null || true
