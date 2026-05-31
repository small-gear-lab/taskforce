<!-- Copyright (c) 2026- Masaki Ishii
Copyright (c) 2026- Small Gear Lab
SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Task Model

## Goal

`taskforce` uses a structured task model that supports:

- a stable core schema for CLI, web UI, and future MCP tooling
- annotations as first-class child records
- plugin-defined extra fields without schema migrations for every workflow

The current implementation is local-first and single-user.

The web UI is intentionally read-only. Mutating operations belong to the CLI and future machine-facing interfaces, not to the browser UI.

## Top-level shape

Each task consists of:

- `id`
- `uuid`
- `core`
- `annotations`
- `extra`

Current Rust shape:

```rust
pub struct Task {
    pub id: Option<u64>,
    pub uuid: String,
    pub core: CoreTaskFields,
    pub annotations: Vec<Annotation>,
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

## Core fields

Core fields are stable and assumed by the CLI, list pages, search, and detail pages.

```rust
pub struct CoreTaskFields {
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_date: Option<NaiveDate>,
    pub deadline: Option<NaiveDate>,
    pub launch_date: Option<NaiveDate>,
    pub target_time_hint: Option<String>,
    pub deadline_time_hint: Option<String>,
    pub launch_time_hint: Option<String>,
    pub project: Option<String>,
    pub tags: Vec<String>,
}
```

### Task status

Current statuses:

- `unstarted`
- `active`
- `waiting`
- `suspended`
- `done`
- `abandoned`
- `mistaken`
- `duplicated`

Operationally:

- open task views focus on `active`, `unstarted`, `waiting`, and `suspended`
- terminal statuses remain visible in all-task views and status-specific views

## Annotations

Annotations are structured records attached to tasks.

```rust
pub struct Annotation {
    pub created_at: DateTime<Utc>,
    pub kind: AnnotationKind,
    pub body: String,
}
```

Current kinds:

- `note`
- `progress`
- `decision`
- `handover`

Annotations are part of the standard model rather than being folded into `description`.

## Extra fields

`extra` stores plugin-defined or user-defined structured data as nested JSON values.

Examples:

- `chatwork.requester`
- `chatwork.request_url`
- `git.repository`
- `git.working_branch`
- `github-repo.repository_url`

Rules:

- core fields stay in `core`
- extra fields stay in `extra`
- dotted CLI keys such as `git.repository` are stored as nested JSON objects
- manifest-defined plugin fields control UI visibility

## Plugin field model

Plugins define fields through manifests under:

```text
plugins/<plugin-id>/manifest.toml
```

Each field defines at least:

- `path`
- `label`
- `placement`

Current placement meanings:

- `left`
- `right`
- `hidden`

### Runtime rules

- only plugins present in `plugins/` are considered active
- plugins without an active manifest are treated as invalid in the UI
- plugin fields not present in the manifest are ignored
- plugin fields are not auto-promoted into core presentation even if names happen to match

## UI presentation model

### List views

Open and filtered lists use a compact list item DTO with:

- title
- status
- schedule metadata
- urgency
- `description_preview`
- tags

`description_preview` is derived server-side and truncated by grapheme cluster.

### Detail views

The detail view is manifest-driven:

- core rows are shown only when values exist
- plugin rows are shown only when values exist and the field is declared in the active manifest
- `left` and `right` plugin sections are rendered generically
- related plugins may be grouped through manifest group metadata

There is no plugin-specific special rendering path anymore. Chatwork, Git, and GitHub data all follow the same manifest rules.

## Search model

The current search system accepts SQL-like `WHERE` fragments and compiles them into backend-specific queries.

Supported field categories:

- core fields such as `status`, `title`, `project`, `deadline`
- tag membership
- plugin dotted paths such as `chatwork.requester`

Supported operators include:

- comparison operators
- `in`
- `between`
- `like`
- `is null`
- boolean `and` / `or` / `not`

The web UI search page builds these conditions from:

- free word query
- status checkboxes
- tag input
- optional raw `WHERE` clauses

## Backends

The task model is implemented behind `TaskBackend`.

Current backends:

- SQLite
- Postgres

Both backends store:

- core task records
- annotation records
- JSON extra payloads

The backend choice is configuration-driven and profile-aware.

## Machine-facing DTO direction

The current system already exposes stable JSON-friendly DTOs for:

- list-style views
- task detail views

This is the intended bridge toward future automation and MCP server work.

The design goal is:

- one structured internal task model
- one stable machine-facing JSON shape
- multiple transports on top of that

## Non-goals

The current task model is not trying to solve:

- multi-user sync
- server-side collaborative editing
- arbitrary schema validation languages
- plugin execution sandboxes

The focus remains a local structured task workflow with extensible metadata.
