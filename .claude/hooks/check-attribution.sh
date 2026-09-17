#!/bin/sh
# Copyright (c) 2026- Masaki Ishii
# Copyright (c) 2026- Small Gear Lab
# SPDX-License-Identifier: MIT OR Apache-2.0

# Claude Code PreToolUse hook body (matcher: Bash, if: "Bash(git commit*)").
# Receives the hook JSON on stdin; denies the Bash call if the git-commit
# command contains a prohibited attribution line.
cmd=$(jq -r '.tool_input.command')
if printf '%s\n' "$cmd" | grep -qiE 'Co-Authored-By|Claude-Session'; then
  printf '%s' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"Co-Authored-By/Claude-Session lines are prohibited in this repo (see CLAUDE.md). Remove them before committing."}}'
fi
