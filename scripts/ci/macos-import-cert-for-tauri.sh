#!/usr/bin/env bash
# Import a Developer ID Application .p12 into a CI keychain for codesign + notarization.
# Required env: APPLE_CERTIFICATE (base64 .p12), APPLE_CERTIFICATE_PASSWORD, KEYCHAIN_PASSWORD
# Optional: GITHUB_ENV — appends APPLE_SIGNING_IDENTITY for later steps.
#
# You must use an Apple "Developer ID Application" certificate (not "Apple Development")
# for distributing installers outside the Mac App Store.

set -euo pipefail

if [[ -z "${APPLE_CERTIFICATE:-}" || -z "${APPLE_CERTIFICATE_PASSWORD:-}" || -z "${KEYCHAIN_PASSWORD:-}" ]]; then
  echo "Missing APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, or KEYCHAIN_PASSWORD" >&2
  exit 1
fi

echo "$APPLE_CERTIFICATE" | base64 -d > certificate.p12

security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security default-keychain -s build.keychain
security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security set-keychain-settings -t 3600 -u build.keychain
security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign -T /usr/bin/security
# Non-interactive builds need partition access; some runner/OS combos return non-zero here even when signing works.
if ! security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" build.keychain 2>/dev/null; then
  echo "warning: set-key-partition-list failed (continuing; check codesign if signing fails)" >&2
fi

rm -f certificate.p12

# grep exits 1 when there is no match; with pipefail that would abort before our message below.
IDENTITY=$(security find-identity -v -p codesigning build.keychain 2>/dev/null \
  | grep "Developer ID Application" \
  | head -1 \
  | awk -F '"' '{print $2}') || true

if [[ -z "$IDENTITY" ]]; then
  echo "No Developer ID Application identity found. Install a Developer ID Application certificate." >&2
  security find-identity -v -p codesigning build.keychain >&2 || true
  exit 1
fi

echo "Using signing identity: $IDENTITY"

if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "APPLE_SIGNING_IDENTITY=$IDENTITY" >> "$GITHUB_ENV"
fi
