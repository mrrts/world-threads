#!/usr/bin/env bash
# Upload Apple signing secrets to GitHub Actions for this repo (never commits secrets).
#
# Prerequisites:
#   - gh CLI installed and authenticated (`gh auth login`)
#   - Developer ID Application certificate exported as .p12 (see Tauri macOS signing docs)
#
# Usage:
#   export APPLE_CERTIFICATE_PASSWORD='your-p12-export-password'
#   export KEYCHAIN_PASSWORD='any-long-random-string'
#   export APPLE_ID='your@apple.id.email'
#   export APPLE_PASSWORD='your-app-specific-password'
#   export APPLE_TEAM_ID='XXXXXXXXXX'
#   Optional: CERT_PATH=/path/to/cert.p12   (default: single *.p12 under repo certs/)
#
#   ./scripts/ci/set-github-apple-secrets.sh
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

REPO="${GITHUB_REPOSITORY:-mrrts/WorldThreads}"

if [[ -n "${CERT_PATH:-}" ]]; then
  P12="$CERT_PATH"
else
  shopt -s nullglob
  files=(certs/*.p12)
  shopt -u nullglob
  if [[ ${#files[@]} -eq 0 ]]; then
    echo "No certs/*.p12 found. Put your exported Developer ID .p12 in certs/ or set CERT_PATH." >&2
    exit 1
  fi
  if [[ ${#files[@]} -gt 1 ]]; then
    echo "Multiple certs/*.p12 files; set CERT_PATH to one of them:" >&2
    printf '  %s\n' "${files[@]}" >&2
    exit 1
  fi
  P12="${files[0]}"
fi

for var in APPLE_CERTIFICATE_PASSWORD KEYCHAIN_PASSWORD APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID; do
  if [[ -z "${!var:-}" ]]; then
    echo "Missing required env: $var" >&2
    exit 1
  fi
done

if [[ ! -f "$P12" ]]; then
  echo "Certificate file not found: $P12" >&2
  exit 1
fi

echo "Using certificate: $P12"
echo "Target repo: $REPO"

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

base64 -i "$P12" | tr -d '\n' > "$TMP"
gh secret set APPLE_CERTIFICATE --repo "$REPO" < "$TMP"

printf '%s' "$APPLE_CERTIFICATE_PASSWORD" | gh secret set APPLE_CERTIFICATE_PASSWORD --repo "$REPO"
printf '%s' "$KEYCHAIN_PASSWORD"             | gh secret set KEYCHAIN_PASSWORD --repo "$REPO"
printf '%s' "$APPLE_ID"                      | gh secret set APPLE_ID --repo "$REPO"
printf '%s' "$APPLE_PASSWORD"               | gh secret set APPLE_PASSWORD --repo "$REPO"
printf '%s' "$APPLE_TEAM_ID"                | gh secret set APPLE_TEAM_ID --repo "$REPO"

echo "Done. Verify under Settings → Secrets and variables → Actions (values are hidden)."
