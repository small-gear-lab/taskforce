<!-- Copyright (c) 2026- Masaki Ishii
Copyright (c) 2026- Small Gear Lab
SPDX-License-Identifier: MIT OR Apache-2.0 -->

# taskforce

`taskforce` is a personal task management system built around:

- a Rust CLI
- a local read-only web UI
- a structured task store
- manifest-driven plugin fields

The project is optimized for single-user local operation first. It supports both SQLite and Postgres backends, including Supabase-hosted Postgres.

## What it does

- stores tasks with explicit core fields and structured extra fields
- tracks annotations such as notes and progress updates
- exposes a local web UI for browsing, searching, and inspecting tasks
- lets plugins define extra fields and UI placement through manifests
- keeps machine-friendly JSON output available from the CLI

## Web UI policy

The web UI is read-only by design.

Task creation, editing, status changes, annotations, and all other mutating operations must go through the CLI or future machine-facing interfaces, not the browser UI.

## Current feature set

### CLI

- `list`
- `next`
- `show`
- `add`
- `edit`
- `status`
- `note`
- `set`
- `get`
- `unset`
- `search`
- `serve`

### Web UI

- open task list
- all task list
- status pages
- tag pages
- search page
- task detail page
- navigation drawer

### Storage

- SQLite backend
- Postgres backend
- profile-based environment selection
- Supabase-compatible TLS verification

### Plugins

- manifest-driven custom fields
- plugin i18n catalogs
- bundled example plugins under `examples/plugins/`
- runtime opt-in through `plugins/<plugin-id>/`

## Local setup

### Config files

- sample config: `config/config.toml.sample`
- sample env file: `config/taskforce.env.sample`

Base files:

- `$XDG_CONFIG_HOME/taskforce/config.toml`
- `$XDG_CONFIG_HOME/taskforce/taskforce.env`

If `XDG_CONFIG_HOME` is unset, taskforce uses `~/.config/taskforce/`.

Taskforce always loads the base files first.

### Profiles

Profiles are declared in `config.toml`:

```toml
[profiles]
default = "production"

[[profiles.items]]
id = "production"
config = "config.production.toml"
env_file = "production.env"

[[profiles.items]]
id = "development"
config = "config.development.toml"
env_file = "development.env"
```

Profile resolution order:

1. `--env <profile>`
2. `TASKFORCE_ENV`
3. `profiles.default`

Relative `config` and `env_file` paths are resolved from the taskforce config directory.

### Backends

Select the backend with:

```toml
[backend]
kind = "sqlite"
```

or:

```toml
[backend]
kind = "postgres"
```

Supported backend kinds:

- `sqlite`
- `postgres`

If SQLite paths are not configured, taskforce uses:

- `$XDG_DATA_HOME/taskforce/taskforce.db`
- or `~/.local/share/taskforce/taskforce.db`

For Postgres, taskforce accepts either:

- `TASKFORCE_POSTGRES_URL`
- `[backend].postgres_url`

or split environment variables:

- `TASKFORCE_POSTGRES_HOST` (required)
- `TASKFORCE_POSTGRES_PASS` (required)
- `TASKFORCE_POSTGRES_USER` (default `postgres`)
- `TASKFORCE_POSTGRES_PORT` (default `5432`)
- `TASKFORCE_POSTGRES_DB` (default `postgres`)
- `TASKFORCE_POSTGRES_SSLMODE` (default `require`)

When TLS verification needs an explicit CA file, set:

- `TASKFORCE_POSTGRES_SSL_ROOT_CERT`
- or `[backend].postgres_ssl_root_cert`

## Plugins

Runtime plugin manifests are loaded only from:

```text
plugins/<plugin-id>/manifest.toml
```

The repository provides bundled examples under:

```text
examples/plugins/
```

Those bundled plugins are not active until you copy or symlink them into `plugins/`.

Current bundled examples:

- `chatwork`
- `git`
- `github-repo`
- `github-issue`
- `github-pr`

`community/plugins/` is reserved for future external plugin contributions and is not scanned at runtime.

## Common commands

```bash
cargo run -- list
cargo run -- next
cargo run -- show 12
cargo run -- show 12 --json
cargo run -- add "Rewrite spec" --status waiting --deadline 2026-06-05 --project taskforce --tag docs
cargo run -- edit 12 "Rewrite spec v2" --clear-deadline --target-date 2026-06-03
cargo run -- status 12
cargo run -- status 12 active
cargo run -- note 12 "Waiting on design handoff" --kind progress
cargo run -- set 12 git.repository '"marie-222/taskforce"'
cargo run -- get 12 git.repository
cargo run -- unset 12 git.repository
cargo run -- search --where "status in ('active', 'waiting')"
cargo run -- search --where "chatwork.requester = '石井'" --json
cargo run -- serve
```

## Search syntax

`taskforce search` accepts repeated SQL-like `WHERE` clauses:

```bash
taskforce search --where "status = 'active'"
taskforce search --where "status in ('active', 'waiting')"
taskforce search --where "deadline between '2026-06-01' and '2026-06-30'"
taskforce search --where "chatwork.requester = '石井'"
taskforce search --where "(status = 'active' or status = 'waiting') and tag = 'release'"
```

Supported operators include:

- `=`
- `!=`
- `<`
- `<=`
- `>`
- `>=`
- `in (...)`
- `between ... and ...`
- `like`
- `is null`
- `is not null`
- `and`
- `or`
- `not`

## JSON output

Machine-friendly JSON output is available for:

- `list --json`
- `show --json`
- `search --json`

The CLI and web API share the same core DTO shapes where possible.

## Development seed data

The development seeder lives at:

```text
scripts/dev/seed_dummy_tasks.py
```

It builds and runs a dev-only Rust bulk seeder.

Defaults:

- `--env development`
- `--count 24`
- `--seed 42`
- `--start 1`

Examples:

```bash
scripts/dev/seed_dummy_tasks.py
scripts/dev/seed_dummy_tasks.py --env development --seed 77 --start 301 --count 300
target/debug/taskforce --env=development search --where "project = 'seed-77'"
```

## systemd user services

Sample units:

- `config/systemd/taskforce.service`
- `config/systemd/taskforce-debug.service`
- `config/systemd/taskforce-development-debug.service`

Typical release setup:

```bash
cargo build --release
systemctl --user daemon-reload
systemctl --user enable --now taskforce.service
```

Typical local development setup:

```bash
cargo build
systemctl --user restart taskforce-debug.service
systemctl --user restart taskforce-development-debug.service
```

When Rust code or embedded assets change, always rebuild before restarting the debug services.

## Related docs

- [docs/task-model.md](docs/task-model.md)

## License

Licensed under either of the following, at your option:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
