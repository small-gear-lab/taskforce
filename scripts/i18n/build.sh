#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
LOCALE_DIR="$ROOT_DIR/locale"

if ! command -v msgfmt >/dev/null 2>&1; then
  echo "msgfmt is required but was not found in PATH." >&2
  exit 1
fi

find "$LOCALE_DIR" -type f -name '*.po' | while read -r po_file; do
  mo_file="${po_file%.po}.mo"
  mkdir -p "$(dirname "$mo_file")"
  msgfmt --check --output-file="$mo_file" "$po_file"
  echo "built $(realpath --relative-to="$ROOT_DIR" "$mo_file")"
done
