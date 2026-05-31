#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
if ! command -v msgfmt >/dev/null 2>&1; then
  echo "msgfmt is required but was not found in PATH." >&2
  exit 1
fi

find "$ROOT_DIR" -type f -path '*/locale/*/LC_MESSAGES/*.po' | while read -r po_file; do
  mo_file="${po_file%.po}.mo"
  mkdir -p "$(dirname "$mo_file")"
  msgfmt --check --output-file="$mo_file" "$po_file"
  echo "built $(realpath --relative-to="$ROOT_DIR" "$mo_file")"
done
