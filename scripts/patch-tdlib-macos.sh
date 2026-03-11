#!/usr/bin/env bash
# Apply TDLib fix for macOS: UserId constructor rejects 'long'; force int64 via LL suffix.
# Run from the tgt repo root: ./scripts/patch-tdlib-macos.sh <path-to-td-clone>
# Example: ./scripts/patch-tdlib-macos.sh td   (when td is the cloned tdlib repo)
set -e
TD_ROOT="${1:-.}"
FILE="${TD_ROOT}/td/telegram/StarManager.cpp"
if [[ ! -f "$FILE" ]]; then
  echo "patch-tdlib-macos.sh: $FILE not found (is TD_ROOT correct?)" >&2
  exit 1
fi
# Idempotent: replace only if not already patched
if grep -q '5001167034LL' "$FILE"; then
  echo "patch-tdlib-macos.sh: already patched"
  exit 0
fi
# Portable in-place sed (BSD sed on macOS requires space after -i)
case "$(uname -s)" in
  Darwin) sed -i '' 's/UserId(G()->is_test_dc() ? 5001167034 : 8353936423)/UserId(G()->is_test_dc() ? 5001167034LL : 8353936423LL)/' "$FILE" ;;
  *)      sed -i 's/UserId(G()->is_test_dc() ? 5001167034 : 8353936423)/UserId(G()->is_test_dc() ? 5001167034LL : 8353936423LL)/' "$FILE" ;;
esac
echo "patch-tdlib-macos.sh: patched $FILE"
