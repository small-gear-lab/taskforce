use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Map;

use crate::backend::{NewTaskInput, Task, TaskBackend, TaskStatus, UpdateTaskInput};

#[derive(Debug, Clone)]
pub struct LocalBackend {
    db_path: PathBuf,
}

impl LocalBackend {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let backend = Self { db_path };
        backend.init_schema()?;
        Ok(backend)
    }

    fn init_schema(&self) -> Result<()> {
        let connection = self.connection()?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              uuid TEXT NOT NULL UNIQUE,
              title TEXT NOT NULL,
              status TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              target_date TEXT,
              deadline TEXT,
              launch_date TEXT,
              target_time_hint TEXT,
              deadline_time_hint TEXT,
              launch_time_hint TEXT,
              project TEXT,
              tags_json TEXT NOT NULL,
              extra_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS task_annotations (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              task_id INTEGER NOT NULL,
              created_at TEXT NOT NULL,
              kind TEXT NOT NULL,
              body TEXT NOT NULL,
              FOREIGN KEY(task_id) REFERENCES tasks(id)
            );
            "#,
        )?;
        Ok(())
    }

    fn connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
            .with_context(|| format!("failed to open {}", self.db_path.display()))
    }

    fn fetch_task(&self, id: u64) -> Result<Task> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            r#"
            SELECT
              id,
              uuid,
              title,
              status,
              created_at,
              updated_at,
              target_date,
              deadline,
              launch_date,
              target_time_hint,
              deadline_time_hint,
              launch_time_hint,
              project,
              tags_json,
              extra_json
            FROM tasks
            WHERE id = ?1
            "#,
        )?;

        statement
            .query_row(params![id], map_task_row)
            .optional()?
            .ok_or_else(|| anyhow!("task {id} was not found"))
    }
}

impl TaskBackend for LocalBackend {
    fn list_pending(&self) -> Result<Vec<Task>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            r#"
            SELECT
              id,
              uuid,
              title,
              status,
              created_at,
              updated_at,
              target_date,
              deadline,
              launch_date,
              target_time_hint,
              deadline_time_hint,
              launch_time_hint,
              project,
              tags_json,
              extra_json
            FROM tasks
            WHERE status IN ('pending', 'active', 'waiting')
            ORDER BY
              CASE WHEN deadline IS NULL THEN 1 ELSE 0 END,
              deadline ASC,
              CASE WHEN target_date IS NULL THEN 1 ELSE 0 END,
              target_date ASC,
              created_at ASC
            "#,
        )?;

        let rows = statement.query_map([], map_task_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn add(&self, input: NewTaskInput) -> Result<Task> {
        let mut task = Task::new(None, generate_local_uuid(), input.title);
        task.core.target_date = input.target_date;
        task.core.deadline = input.deadline;
        task.core.launch_date = input.launch_date;
        task.core.target_time_hint = input.target_time_hint;
        task.core.deadline_time_hint = input.deadline_time_hint;
        task.core.launch_time_hint = input.launch_time_hint;
        task.core.project = input.project;
        task.core.tags = input.tags;
        let connection = self.connection()?;
        let tags_json = serde_json::to_string(&task.core.tags)?;
        let extra_json = serde_json::to_string(&task.extra)?;

        connection.execute(
            r#"
            INSERT INTO tasks (
              uuid,
              title,
              status,
              created_at,
              updated_at,
              target_date,
              deadline,
              launch_date,
              target_time_hint,
              deadline_time_hint,
              launch_time_hint,
              project,
              tags_json,
              extra_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                task.uuid,
                task.core.title,
                task_status_text(task.core.status),
                task.core.created_at.to_rfc3339(),
                task.core.updated_at.to_rfc3339(),
                task.core.target_date.map(|value| value.to_string()),
                task.core.deadline.map(|value| value.to_string()),
                task.core.launch_date.map(|value| value.to_string()),
                task.core.target_time_hint,
                task.core.deadline_time_hint,
                task.core.launch_time_hint,
                task.core.project,
                tags_json,
                extra_json,
            ],
        )?;

        task.id = Some(connection.last_insert_rowid() as u64);
        Ok(task)
    }

    fn edit(&self, id: u64, input: UpdateTaskInput) -> Result<Task> {
        let connection = self.connection()?;
        let now = Utc::now().to_rfc3339();
        let mut task = self.fetch_task(id)?;

        if let Some(title) = input.title {
            task.core.title = title;
        }
        if input.clear_target_date {
            task.core.target_date = None;
        }
        if let Some(target_date) = input.target_date {
            task.core.target_date = Some(target_date);
        }
        if input.clear_deadline {
            task.core.deadline = None;
        }
        if let Some(deadline) = input.deadline {
            task.core.deadline = Some(deadline);
        }
        if input.clear_launch_date {
            task.core.launch_date = None;
        }
        if let Some(launch_date) = input.launch_date {
            task.core.launch_date = Some(launch_date);
        }
        if input.clear_target_time_hint {
            task.core.target_time_hint = None;
        }
        if let Some(target_time_hint) = input.target_time_hint {
            task.core.target_time_hint = Some(target_time_hint);
        }
        if input.clear_deadline_time_hint {
            task.core.deadline_time_hint = None;
        }
        if let Some(deadline_time_hint) = input.deadline_time_hint {
            task.core.deadline_time_hint = Some(deadline_time_hint);
        }
        if input.clear_launch_time_hint {
            task.core.launch_time_hint = None;
        }
        if let Some(launch_time_hint) = input.launch_time_hint {
            task.core.launch_time_hint = Some(launch_time_hint);
        }
        if input.clear_project {
            task.core.project = None;
        }
        if let Some(project) = input.project {
            task.core.project = Some(project);
        }
        if input.clear_tags {
            task.core.tags = Vec::new();
        }
        if let Some(tags) = input.tags {
            task.core.tags = tags;
        }

        let tags_json = serde_json::to_string(&task.core.tags)?;
        let updated = connection.execute(
            r#"
            UPDATE tasks
            SET title = ?1,
                updated_at = ?2,
                target_date = ?3,
                deadline = ?4,
                launch_date = ?5,
                target_time_hint = ?6,
                deadline_time_hint = ?7,
                launch_time_hint = ?8,
                project = ?9,
                tags_json = ?10
            WHERE id = ?11
            "#,
            params![
                task.core.title,
                now,
                task.core.target_date.map(|value| value.to_string()),
                task.core.deadline.map(|value| value.to_string()),
                task.core.launch_date.map(|value| value.to_string()),
                task.core.target_time_hint,
                task.core.deadline_time_hint,
                task.core.launch_time_hint,
                task.core.project,
                tags_json,
                id,
            ],
        )?;

        if updated == 0 {
            return Err(anyhow!("task {id} was not found"));
        }

        self.fetch_task(id)
    }

    fn get_task(&self, id: u64) -> Result<Task> {
        self.fetch_task(id)
    }

    fn set_extra(&self, id: u64, key: &str, value: serde_json::Value) -> Result<Task> {
        let connection = self.connection()?;
        let mut task = self.fetch_task(id)?;
        task.extra.insert(key.to_string(), value);

        connection.execute(
            "UPDATE tasks SET extra_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![
                serde_json::to_string(&task.extra)?,
                Utc::now().to_rfc3339(),
                id
            ],
        )?;

        self.fetch_task(id)
    }

    fn get_extra(&self, id: u64, key: &str) -> Result<Option<serde_json::Value>> {
        let task = self.fetch_task(id)?;
        Ok(task.extra.get(key).cloned())
    }

    fn unset_extra(&self, id: u64, key: &str) -> Result<Task> {
        let connection = self.connection()?;
        let mut task = self.fetch_task(id)?;
        task.extra.remove(key);

        connection.execute(
            "UPDATE tasks SET extra_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![
                serde_json::to_string(&task.extra)?,
                Utc::now().to_rfc3339(),
                id
            ],
        )?;

        self.fetch_task(id)
    }

    fn delete(&self, id: u64) -> Result<Task> {
        update_task_status(self, id, TaskStatus::Deleted)
    }

    fn mark_done(&self, id: u64) -> Result<Task> {
        update_task_status(self, id, TaskStatus::Done)
    }

    fn next_task(&self) -> Result<Option<Task>> {
        Ok(self.list_pending()?.into_iter().next())
    }
}

fn update_task_status(backend: &LocalBackend, id: u64, status: TaskStatus) -> Result<Task> {
    let connection = backend.connection()?;
    let updated = connection.execute(
        "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![task_status_text(status), Utc::now().to_rfc3339(), id],
    )?;

    if updated == 0 {
        return Err(anyhow!("task {id} was not found"));
    }

    backend.fetch_task(id)
}

fn map_task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let tags_json: String = row.get(13)?;
    let extra_json: String = row.get(14)?;

    let tags = serde_json::from_str(&tags_json).map_err(json_decode_error)?;
    let extra: Map<String, serde_json::Value> =
        serde_json::from_str(&extra_json).map_err(json_decode_error)?;

    Ok(Task {
        id: Some(row.get::<_, i64>(0)? as u64),
        uuid: row.get(1)?,
        core: crate::backend::CoreTaskFields {
            title: row.get(2)?,
            status: parse_task_status(&row.get::<_, String>(3)?),
            created_at: parse_datetime(&row.get::<_, String>(4)?)?,
            updated_at: parse_datetime(&row.get::<_, String>(5)?)?,
            target_date: parse_optional_date(row.get(6)?)?,
            deadline: parse_optional_date(row.get(7)?)?,
            launch_date: parse_optional_date(row.get(8)?)?,
            target_time_hint: row.get(9)?,
            deadline_time_hint: row.get(10)?,
            launch_time_hint: row.get(11)?,
            project: row.get(12)?,
            tags,
        },
        annotations: Vec::new(),
        extra,
    })
}

fn parse_datetime(value: &str) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(json_decode_error)
}

fn parse_optional_date(value: Option<String>) -> rusqlite::Result<Option<NaiveDate>> {
    value
        .map(|text| NaiveDate::parse_from_str(&text, "%Y-%m-%d").map_err(json_decode_error))
        .transpose()
}

fn json_decode_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn task_status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::Active => "active",
        TaskStatus::Waiting => "waiting",
        TaskStatus::Done => "done",
        TaskStatus::Deleted => "deleted",
    }
}

fn parse_task_status(value: &str) -> TaskStatus {
    match value {
        "pending" => TaskStatus::Pending,
        "active" => TaskStatus::Active,
        "waiting" => TaskStatus::Waiting,
        "done" => TaskStatus::Done,
        "deleted" => TaskStatus::Deleted,
        _ => TaskStatus::Pending,
    }
}

fn generate_local_uuid() -> String {
    format!(
        "local-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::LocalBackend;
    use crate::backend::{NewTaskInput, TaskBackend, UpdateTaskInput};

    #[test]
    fn add_and_list_pending_tasks() -> Result<()> {
        let backend = LocalBackend::new(unique_db_path("taskforce-local-backend"))?;

        let added = backend.add(NewTaskInput {
            title: "Ship SQLite backend".into(),
            deadline: Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 5).expect("date")),
            tags: vec!["release".into()],
            ..Default::default()
        })?;
        let tasks = backend.list_pending()?;

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, added.id);
        assert_eq!(tasks[0].title(), "Ship SQLite backend");
        assert_eq!(
            tasks[0].core.deadline,
            Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 5).expect("date"))
        );
        assert_eq!(tasks[0].core.tags, vec!["release"]);
        Ok(())
    }

    #[test]
    fn edit_done_and_delete_task() -> Result<()> {
        let backend = LocalBackend::new(unique_db_path("taskforce-local-backend-status"))?;

        let added = backend.add(NewTaskInput {
            title: "Old title".into(),
            ..Default::default()
        })?;
        let edited = backend.edit(
            added.id.expect("id"),
            UpdateTaskInput {
                title: Some("New title".into()),
                project: Some("taskforce".into()),
                tags: Some(vec!["ops".into()]),
                ..Default::default()
            },
        )?;
        assert_eq!(edited.title(), "New title");
        assert_eq!(edited.core.project.as_deref(), Some("taskforce"));
        assert_eq!(edited.core.tags, vec!["ops"]);

        let done = backend.mark_done(added.id.expect("id"))?;
        assert_eq!(done.core.status, crate::backend::TaskStatus::Done);

        let second = backend.add(NewTaskInput {
            title: "Delete me".into(),
            ..Default::default()
        })?;
        let deleted = backend.delete(second.id.expect("id"))?;
        assert_eq!(deleted.core.status, crate::backend::TaskStatus::Deleted);
        Ok(())
    }

    #[test]
    fn clears_optional_fields_and_updates_extra() -> Result<()> {
        let backend = LocalBackend::new(unique_db_path("taskforce-local-backend-extra"))?;

        let added = backend.add(NewTaskInput {
            title: "Structured task".into(),
            deadline: Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 5).expect("date")),
            project: Some("taskforce".into()),
            ..Default::default()
        })?;
        let id = added.id.expect("id");

        let edited = backend.edit(
            id,
            UpdateTaskInput {
                clear_deadline: true,
                clear_project: true,
                ..Default::default()
            },
        )?;
        assert_eq!(edited.core.deadline, None);
        assert_eq!(edited.core.project, None);

        backend.set_extra(id, "requester", serde_json::Value::String("ishii".into()))?;
        assert_eq!(
            backend.get_extra(id, "requester")?,
            Some(serde_json::Value::String("ishii".into()))
        );

        let edited = backend.unset_extra(id, "requester")?;
        assert!(!edited.extra.contains_key("requester"));
        Ok(())
    }

    fn unique_db_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.db"))
    }
}
