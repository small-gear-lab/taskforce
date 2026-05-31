<!--
Copyright (c) 2026- Masaki Ishii
Copyright (c) 2026- Small Gear Lab
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# AGENTS.md

This file defines rules for this repository.
Repository-level instructions belong here.

## Language

- Respond in Japanese unless a task explicitly requires another language.

## Working style

- Keep answers concise and concrete.

## Copyright headers

- Add a copyright and SPDX header to every tracked source or documentation file by default.
- Exclude generated files and files that are intentionally ignored from manual maintenance.
- Preserve format-specific conventions when adding headers:
  - Markdown: use an HTML comment block.
  - Shell and Python: keep the shebang on the first line and place the header immediately after it.
  - CSS: use a block comment.
- State the conclusion first.
- Prefer practical decisions over speculative architecture.

## Local overrides

- Personal, user-specific instructions may be kept in `AGENTS.override.md`.
- `AGENTS.override.md` must not be committed in this repository.
