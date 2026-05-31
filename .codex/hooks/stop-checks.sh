#!/usr/bin/env bash

# Copyright (c) 2026- Masaki Ishii
# Copyright (c) 2026- Small Gear Lab
# SPDX-License-Identifier: MIT OR Apache-2.0

set -euo pipefail

cargo fmt

if ! cargo clippy --all-targets -- -D warnings; then
  cat >&2 <<'EOF'
Stop hook: clippy reported warnings or errors.
Fix the reported issues, then rerun `cargo clippy --all-targets -- -D warnings` before continuing.
EOF
  exit 1
fi
