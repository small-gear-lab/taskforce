# taskforce

Personal task management system built around a Rust API, a web UI, and Taskwarrior-compatible workflows.

## Goals

- Keep the core repository focused on the product backend.
- Avoid a long-lived monorepo unless there is a clear operational need.
- Treat upstream Taskwarrior as a reference implementation, not as vendored source by default.
- Leave room for a separate web UI repository and infra repository.

## Repository role

This repository is intended to hold the main application backend and domain logic.

- Main product repository: `taskforce`
- Separate frontend repository: `taskforce-web`
- Optional infrastructure repository: `taskforce-infra`

## Planned structure

```text
taskforce/
├── README.md
├── AGENTS.md
├── Cargo.toml
├── .gitignore
├── src/
│   └── main.rs
├── crates/
│   ├── taskcore/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── taskapi/
│       ├── Cargo.toml
│       └── src/
├── docs/
│   ├── architecture.md
│   └── api.md
└── scripts/
    └── dev/
```

## Notes

- `vendor/taskwarrior` is intentionally not included at the beginning.
- If direct source-level comparison with upstream becomes frequent, revisit that decision later.
- Sync storage and Web UI details should be designed to work without forcing a monorepo.
