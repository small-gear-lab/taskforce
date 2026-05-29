# AGENTS.md

## Language

- Respond in Japanese unless a task explicitly requires another language.

## Working style

- Keep answers concise and concrete.
- State the conclusion first.
- Prefer practical decisions over speculative architecture.

## Repository intent

- This repository is the main product backend for `taskforce`.
- Avoid expanding it into a catch-all monorepo unless there is a strong operational reason.
- Treat upstream Taskwarrior as a reference implementation unless vendoring becomes clearly necessary.

## Architecture assumptions

- Main backend: Rust
- Separate Web UI repository is expected
- Separate infrastructure repository is allowed
- Taskwarrior compatibility is a product goal, but direct source embedding is not the default

## Editing policy

- Keep diffs small and intentional.
- Do not introduce broad formatting-only changes unless requested.
- Prefer files and naming that will still make sense if the project grows into multiple repositories.

## Initial repository bias

- Prioritize API and domain design over early UI coupling.
- Keep references to external upstream projects documented, not copied, unless there is a clear benefit.
