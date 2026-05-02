#!/usr/bin/env bash
# Import a Developer ID Application .p12 into a CI keychain for codesign + notarization.
# Required env: APPLE_CERTIFICATE (base64 .p12), APPLE_CERTIFICATE_PASSWORD, KEYCHAIN_PASSWORD
# Optional: GITHUB_ENV — appends APPLE_SIGNING_IDENTITY for later steps.
#
# You must use an Apple "Developer ID Application" certificate (not "Apple Development")
# for distributing installers outside the Mac App Store.

set -euo pipefail

if [[ -z "${APPLE_CERTIFICATE:-}" || -z "${APPLE_CERTIFICATE_PASSWORD:-}" || -z "${KEYCHAIN_PASSWORD:-}" ]]; then
  echo "::error::macos-import-cert: missing APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, or KEYCHAIN_PASSWORD" >&2
  exit 1
fi

echo "::group::Apple signing — decode .p12 and create keychain"
echo "$APPLE_CERTIFICATE" | base64 -d > certificate.p12

security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security default-keychain -s build.keychain
security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security set-keychain-settings -t 3600 -u build.keychain
echo "macos-import-cert: importing certificate into build.keychain..."
security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign -T /usr/bin/security
echo "::endgroup::"

echo "::group::Apple signing — key partition list (codesign may need this on the runner)"
# Non-interactive builds need partition access; some runner/OS combos return non-zero here even when signing works.
PART_ERR="$(mktemp)"
if ! security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" build.keychain 2>"$PART_ERR"; then
  echo "warning: set-key-partition-list failed (continuing; check codesign if signing fails)" >&2
  if [[ -s "$PART_ERR" ]]; then
    echo "set-key-partition-list stderr:" >&2
    cat "$PART_ERR" >&2
  fi
fi
rm -f "$PART_ERR"
echo "::endgroup::"

rm -f certificate.p12

echo "::group::Apple signing — resolve Developer ID Application identity"
# grep exits 1 when there is no match; with pipefail that would abort before our message below.
IDENTITY=$(security find-identity -v -p codesigning build.keychain 2>/dev/null \
  | grep "Developer ID Application" \
  | head -1 \
  | awk -F '"' '{print $2}') || true

if [[ -z "$IDENTITY" ]]; then
  echo "::error::No Developer ID Application identity found after import." >&2
  echo "Most common cause: APPLE_CERTIFICATE decodes to a Mac Development or Apple Development .p12. Distribution builds need a Developer ID Application certificate from Apple (export .p12 with private key)." >&2
  echo "--- security find-identity -v -p codesigning build.keychain ---" >&2
  security find-identity -v -p codesigning build.keychain >&2 || true
  echo "::endgroup::" >&2 || true
  exit 1
fi
echo "::endgroup::"

echo "Using signing identity: $IDENTITY"

if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "APPLE_SIGNING_IDENTITY=$IDENTITY" >> "$GITHUB_ENV"
fi
