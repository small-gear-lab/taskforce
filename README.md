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

## Local setup

- Install Taskwarrior separately and make the `task` binary available either via `PATH` or `task_bin` in config.
- A sample config is available at `config/config.toml.sample`.
- Copy it to `$XDG_CONFIG_HOME/taskforce/config.toml`, or `~/.config/taskforce/config.toml` if `XDG_CONFIG_HOME` is unset.

## Current commands

```bash
cargo run -- list
cargo run -- add "Write docs"
cargo run -- edit 1 "Write better docs"
cargo run -- delete 1
cargo run -- done 1
TASKFORCE_TASK_BIN="$HOME/.local/opt/taskwarrior/bin/task" cargo run -- serve
```

- `serve` binds to `127.0.0.1` and chooses a free port unless a host or port is configured.
