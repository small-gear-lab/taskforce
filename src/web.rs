use std::net::SocketAddr;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router, http::StatusCode};

use crate::backend::{Task, TaskBackend};

pub async fn serve<B>(backend: B, addr: SocketAddr) -> Result<()>
where
    B: TaskBackend + Clone + Send + Sync + 'static,
{
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    println!("serving taskforce at http://{local_addr}");
    axum::serve(listener, app_router(backend)).await?;
    Ok(())
}

pub fn app_router<B>(backend: B) -> Router
where
    B: TaskBackend + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(index))
        .route("/api/tasks", get(api_tasks::<B>))
        .route("/api/tasks/{id}", get(api_task::<B>))
        .route("/tasks/{id}", get(task_detail))
        .with_state(backend)
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn api_tasks<B>(State(backend): State<B>) -> Result<Json<Vec<Task>>, axum::http::StatusCode>
where
    B: TaskBackend + Clone + Send + Sync + 'static,
{
    backend
        .list_pending()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn api_task<B>(
    Path(id): Path<u64>,
    State(backend): State<B>,
) -> Result<Json<Task>, StatusCode>
where
    B: TaskBackend + Clone + Send + Sync + 'static,
{
    backend
        .get_task(id)
        .map(Json)
        .map_err(map_task_error_status)
}

async fn task_detail(Path(_id): Path<u64>) -> Html<&'static str> {
    Html(DETAIL_HTML)
}

fn map_task_error_status(error: anyhow::Error) -> StatusCode {
    if error.to_string().contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>taskforce</title>
    <style>
      :root {
        --bg: #f4efe7;
        --ink: #1a1816;
        --muted: #6c625a;
        --card: rgba(255, 252, 246, 0.84);
        --line: rgba(26, 24, 22, 0.12);
        --accent: #c4532f;
        --accent-2: #21453d;
      }

      * {
        box-sizing: border-box;
      }

      body {
        margin: 0;
        min-height: 100vh;
        font-family: "Iowan Old Style", "Palatino Linotype", serif;
        color: var(--ink);
        background:
          radial-gradient(circle at top left, rgba(196, 83, 47, 0.16), transparent 28%),
          radial-gradient(circle at bottom right, rgba(33, 69, 61, 0.18), transparent 32%),
          linear-gradient(145deg, #efe4d2 0%, var(--bg) 52%, #e5ddd1 100%);
      }

      main {
        width: min(980px, calc(100% - 32px));
        margin: 0 auto;
        padding: 48px 0 72px;
      }

      .hero {
        display: grid;
        gap: 14px;
        margin-bottom: 28px;
      }

      .eyebrow {
        letter-spacing: 0.18em;
        text-transform: uppercase;
        font-size: 12px;
        color: var(--accent-2);
      }

      h1 {
        margin: 0;
        font-size: clamp(42px, 8vw, 78px);
        line-height: 0.92;
        font-weight: 700;
      }

      .lede {
        max-width: 640px;
        margin: 0;
        font-size: 18px;
        line-height: 1.6;
        color: var(--muted);
      }

      .panel {
        border: 1px solid var(--line);
        border-radius: 24px;
        background: var(--card);
        backdrop-filter: blur(14px);
        box-shadow: 0 22px 60px rgba(26, 24, 22, 0.08);
        overflow: hidden;
      }

      .panel-head {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 16px;
        padding: 20px 22px;
        border-bottom: 1px solid var(--line);
      }

      .panel-head h2 {
        margin: 0;
        font-size: 24px;
      }

      button {
        border: 0;
        border-radius: 999px;
        padding: 10px 16px;
        font: inherit;
        color: #fffaf3;
        background: linear-gradient(135deg, var(--accent), #d47936);
        cursor: pointer;
      }

      ul {
        list-style: none;
        margin: 0;
        padding: 10px 0;
      }

      li {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 14px;
        align-items: center;
        padding: 14px 22px;
        border-top: 1px solid rgba(26, 24, 22, 0.08);
      }

      li:first-child {
        border-top: 0;
      }

      .task-id {
        min-width: 40px;
        font-size: 12px;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        color: var(--accent-2);
      }

      .task-desc {
        font-size: 18px;
      }

      .task-link {
        color: inherit;
        text-decoration: none;
        border-bottom: 1px solid transparent;
        transition: border-color 160ms ease, color 160ms ease;
      }

      .task-link:hover {
        color: var(--accent);
        border-color: rgba(196, 83, 47, 0.35);
      }

      .task-urgency {
        color: var(--muted);
        font-size: 14px;
      }

      .empty {
        padding: 24px 22px 28px;
        color: var(--muted);
      }

      @media (max-width: 640px) {
        li {
          grid-template-columns: 1fr;
          gap: 6px;
        }
      }
    </style>
  </head>
  <body>
    <main>
      <section class="hero">
        <div class="eyebrow">Local Task Console</div>
        <h1>taskforce</h1>
        <p class="lede">
          Open tasks from your local taskforce database, served over a tiny local HTTP view.
        </p>
      </section>
      <section class="panel">
        <div class="panel-head">
          <h2>Open Tasks</h2>
          <button id="refresh" type="button">Refresh</button>
        </div>
        <ul id="task-list"></ul>
        <div id="empty" class="empty" hidden>No open tasks.</div>
      </section>
    </main>
    <script>
      const taskList = document.getElementById("task-list");
      const emptyState = document.getElementById("empty");
      const refreshButton = document.getElementById("refresh");

      async function loadTasks() {
        const response = await fetch("/api/tasks");
        const tasks = await response.json();

        taskList.innerHTML = "";
        emptyState.hidden = tasks.length !== 0;

        for (const task of tasks) {
          const item = document.createElement("li");
          item.innerHTML = `
            <span class="task-id">#${task.id ?? "?"}</span>
            <span class="task-desc"><a class="task-link" href="/tasks/${task.id ?? ""}"></a></span>
            <span class="task-urgency">urgency ${Number(task.extra?.urgency ?? 0).toFixed(1)}</span>
          `;
          item.querySelector(".task-link").textContent = task.core.title;
          taskList.appendChild(item);
        }
      }

      refreshButton.addEventListener("click", () => {
        loadTasks().catch(console.error);
      });

      loadTasks().catch(console.error);
    </script>
  </body>
</html>
"#;

const DETAIL_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>taskforce task</title>
    <style>
      :root {
        --bg: #f6f0e4;
        --ink: #171512;
        --muted: #6d645a;
        --paper: rgba(255, 251, 244, 0.92);
        --line: rgba(23, 21, 18, 0.12);
        --accent: #c4532f;
        --accent-2: #21453d;
        --accent-3: #cbb28f;
      }

      * { box-sizing: border-box; }

      body {
        margin: 0;
        min-height: 100vh;
        font-family: "Iowan Old Style", "Palatino Linotype", serif;
        color: var(--ink);
        background:
          linear-gradient(125deg, rgba(33, 69, 61, 0.16), transparent 28%),
          radial-gradient(circle at 20% 0%, rgba(196, 83, 47, 0.18), transparent 24%),
          linear-gradient(180deg, #ede2cf 0%, var(--bg) 46%, #efe8dc 100%);
      }

      main {
        width: min(1100px, calc(100% - 32px));
        margin: 0 auto;
        padding: 42px 0 72px;
      }

      .topline {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 16px;
        margin-bottom: 18px;
      }

      .backlink {
        color: var(--accent-2);
        text-decoration: none;
        font-size: 14px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
      }

      .shell {
        border: 1px solid var(--line);
        border-radius: 28px;
        overflow: hidden;
        background: var(--paper);
        box-shadow: 0 26px 70px rgba(23, 21, 18, 0.08);
      }

      .hero {
        display: grid;
        gap: 16px;
        padding: 28px 28px 26px;
        border-bottom: 1px solid var(--line);
        background:
          linear-gradient(135deg, rgba(255,255,255,0.72), rgba(255,255,255,0.36)),
          linear-gradient(120deg, rgba(196, 83, 47, 0.08), rgba(33, 69, 61, 0.08));
      }

      .meta-line {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
      }

      .chip {
        border: 1px solid rgba(23, 21, 18, 0.12);
        border-radius: 999px;
        padding: 7px 12px;
        font-size: 12px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--accent-2);
        background: rgba(255, 255, 255, 0.64);
      }

      h1 {
        margin: 0;
        font-size: clamp(34px, 5vw, 58px);
        line-height: 0.98;
      }

      .hero-copy {
        margin: 0;
        max-width: 760px;
        color: var(--muted);
        font-size: 18px;
        line-height: 1.65;
      }

      .grid {
        display: grid;
        grid-template-columns: 1.15fr 0.85fr;
        gap: 0;
      }

      .column {
        padding: 24px 28px 32px;
      }

      .column + .column {
        border-left: 1px solid var(--line);
      }

      .section {
        display: grid;
        gap: 14px;
        margin-bottom: 28px;
      }

      .section:last-child {
        margin-bottom: 0;
      }

      .section-label {
        font-size: 12px;
        letter-spacing: 0.18em;
        text-transform: uppercase;
        color: var(--accent-2);
      }

      .section-body {
        margin: 0;
        color: var(--ink);
        font-size: 18px;
        line-height: 1.7;
        white-space: pre-wrap;
      }

      .schedule {
        display: grid;
        gap: 12px;
      }

      .schedule-row {
        display: grid;
        grid-template-columns: 120px 1fr;
        gap: 10px;
        align-items: baseline;
        padding-bottom: 12px;
        border-bottom: 1px solid rgba(23, 21, 18, 0.08);
      }

      .schedule-row:last-child {
        border-bottom: 0;
        padding-bottom: 0;
      }

      .schedule-label {
        color: var(--muted);
        font-size: 14px;
        text-transform: uppercase;
        letter-spacing: 0.08em;
      }

      .schedule-value {
        font-size: 18px;
      }

      .kv-list {
        display: grid;
        gap: 14px;
      }

      .kv-item {
        display: grid;
        gap: 6px;
        padding-bottom: 14px;
        border-bottom: 1px solid rgba(23, 21, 18, 0.08);
      }

      .kv-item:last-child {
        border-bottom: 0;
        padding-bottom: 0;
      }

      .kv-key {
        font-size: 12px;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--muted);
      }

      .kv-value {
        font-size: 17px;
        line-height: 1.6;
        white-space: pre-wrap;
        word-break: break-word;
      }

      .tag-list {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }

      .tag {
        border-radius: 999px;
        background: rgba(33, 69, 61, 0.09);
        color: var(--accent-2);
        padding: 7px 12px;
        font-size: 14px;
      }

      .scope-list {
        display: grid;
        gap: 10px;
      }

      .scope-item {
        padding: 14px 16px;
        border-radius: 18px;
        background: linear-gradient(135deg, rgba(203, 178, 143, 0.18), rgba(255,255,255,0.4));
        border: 1px solid rgba(23, 21, 18, 0.08);
      }

      .scope-item strong {
        display: block;
        margin-bottom: 4px;
        font-size: 16px;
      }

      .scope-item span {
        color: var(--muted);
        font-size: 14px;
      }

      pre {
        margin: 0;
        padding: 18px;
        border-radius: 18px;
        overflow: auto;
        background: #191612;
        color: #f7efe4;
        font-size: 13px;
        line-height: 1.6;
      }

      .empty {
        color: var(--muted);
        font-style: italic;
      }

      @media (max-width: 860px) {
        .grid {
          grid-template-columns: 1fr;
        }

        .column + .column {
          border-left: 0;
          border-top: 1px solid var(--line);
        }

        .schedule-row {
          grid-template-columns: 1fr;
          gap: 4px;
        }
      }
    </style>
  </head>
  <body>
    <main>
      <div class="topline">
        <a class="backlink" href="/">Back to Open Tasks</a>
      </div>
      <section class="shell">
        <header class="hero">
          <div class="meta-line" id="meta-line"></div>
          <h1 id="task-title">Loading task…</h1>
          <p class="hero-copy" id="task-abstract"></p>
        </header>
        <div class="grid">
          <section class="column">
            <div class="section">
              <div class="section-label">Description</div>
              <p class="section-body" id="task-description"></p>
            </div>
            <div class="section">
              <div class="section-label">Scope</div>
              <div class="scope-list" id="scope-list"></div>
              <div class="empty" id="scope-empty" hidden>No scope details.</div>
            </div>
          </section>
          <aside class="column">
            <div class="section">
              <div class="section-label">Schedule</div>
              <div class="schedule" id="schedule"></div>
            </div>
            <div class="section">
              <div class="section-label">Project & Tags</div>
              <div class="kv-list">
                <div class="kv-item">
                  <div class="kv-key">Project</div>
                  <div class="kv-value" id="project-value"></div>
                </div>
                <div class="kv-item">
                  <div class="kv-key">Tags</div>
                  <div class="tag-list" id="tag-list"></div>
                </div>
              </div>
            </div>
            <div class="section">
              <div class="section-label">Chatwork</div>
              <div class="kv-list">
                <div class="kv-item">
                  <div class="kv-key">Requester</div>
                  <div class="kv-value" id="requester-value"></div>
                </div>
                <div class="kv-item">
                  <div class="kv-key">Source</div>
                  <div class="kv-value" id="source-value"></div>
                </div>
                <div class="kv-item">
                  <div class="kv-key">Raw Extra</div>
                  <pre id="raw-extra"></pre>
                </div>
              </div>
            </div>
          </aside>
        </div>
      </section>
    </main>
    <script>
      const taskId = window.location.pathname.split("/").pop();

      function textOrFallback(value, fallback = "—") {
        return value == null || value === "" ? fallback : value;
      }

      function dateLine(date, hint) {
        if (!date && !hint) return "—";
        return [date, hint].filter(Boolean).join(" ");
      }

      async function loadTask() {
        const response = await fetch(`/api/tasks/${taskId}`);
        if (!response.ok) {
          document.getElementById("task-title").textContent = "Task not found";
          document.getElementById("task-abstract").textContent = "The requested task could not be loaded.";
          return;
        }

        const task = await response.json();
        const chatwork = task.extra?.chatwork ?? {};
        const source = chatwork.source ?? {};
        const metaLine = document.getElementById("meta-line");
        const schedule = document.getElementById("schedule");
        const tagList = document.getElementById("tag-list");
        const scopeList = document.getElementById("scope-list");
        const scopeEmpty = document.getElementById("scope-empty");

        document.title = `${task.core.title} | taskforce`;
        document.getElementById("task-title").textContent = task.core.title;
        document.getElementById("task-abstract").textContent = textOrFallback(chatwork.abstract || chatwork.summary, "No abstract yet.");
        document.getElementById("task-description").textContent = textOrFallback(chatwork.description, "No description yet.");
        document.getElementById("project-value").textContent = textOrFallback(task.core.project);
        document.getElementById("requester-value").textContent = textOrFallback(chatwork.requester);
        document.getElementById("raw-extra").textContent = JSON.stringify(task.extra, null, 2);

        const sourceParts = [];
        if (source.url) {
          sourceParts.push(source.url);
        }
        if (source.sender?.name) {
          sourceParts.push(`sender: ${source.sender.name}`);
        }
        if (source.sent_at) {
          sourceParts.push(`sent: ${source.sent_at}`);
        }
        document.getElementById("source-value").textContent = textOrFallback(sourceParts.join("\n"));

        metaLine.innerHTML = "";
        for (const chipText of [
          `#${task.id ?? "?"}`,
          task.core.status,
          task.core.project ?? "no project",
          `${task.annotations?.length ?? 0} annotations`
        ]) {
          const chip = document.createElement("span");
          chip.className = "chip";
          chip.textContent = chipText;
          metaLine.appendChild(chip);
        }

        schedule.innerHTML = "";
        for (const [label, value] of [
          ["Target", dateLine(task.core.target_date, task.core.target_time_hint)],
          ["Deadline", dateLine(task.core.deadline, task.core.deadline_time_hint)],
          ["Launch", dateLine(task.core.launch_date, task.core.launch_time_hint)]
        ]) {
          const row = document.createElement("div");
          row.className = "schedule-row";
          row.innerHTML = `
            <div class="schedule-label">${label}</div>
            <div class="schedule-value">${value}</div>
          `;
          schedule.appendChild(row);
        }

        tagList.innerHTML = "";
        if ((task.core.tags ?? []).length === 0) {
          const empty = document.createElement("div");
          empty.className = "empty";
          empty.textContent = "No tags.";
          tagList.appendChild(empty);
        } else {
          for (const tag of task.core.tags) {
            const item = document.createElement("span");
            item.className = "tag";
            item.textContent = tag;
            tagList.appendChild(item);
          }
        }

        scopeList.innerHTML = "";
        const targetSites = chatwork.target_sites ?? [];
        scopeEmpty.hidden = targetSites.length !== 0;
        for (const site of targetSites) {
          const item = document.createElement("div");
          item.className = "scope-item";
          item.innerHTML = `
            <strong>${site.label ?? "Unknown target"}</strong>
            <span>${[site.site_code, site.raw].filter(Boolean).join(" · ")}</span>
          `;
          scopeList.appendChild(item);
        }
      }

      loadTask().catch(console.error);
    </script>
  </body>
</html>
"#;

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use serde_json::{Map, Value};
    use tower::ServiceExt;

    use crate::backend::{
        CoreTaskFields, NewTaskInput, Task, TaskBackend, TaskStatus, UpdateTaskInput,
    };

    #[derive(Clone)]
    struct MockBackend {
        tasks: Vec<Task>,
    }

    impl TaskBackend for MockBackend {
        fn list_pending(&self) -> anyhow::Result<Vec<Task>> {
            Ok(self.tasks.clone())
        }

        fn add(&self, _input: NewTaskInput) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn edit(&self, _id: u64, _input: UpdateTaskInput) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn get_task(&self, id: u64) -> anyhow::Result<Task> {
            self.tasks
                .iter()
                .find(|task| task.id == Some(id))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("task {id} was not found"))
        }

        fn set_extra(
            &self,
            _id: u64,
            _key: &str,
            _value: serde_json::Value,
        ) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn get_extra(&self, _id: u64, _key: &str) -> anyhow::Result<Option<serde_json::Value>> {
            unreachable!("not used in web tests")
        }

        fn unset_extra(&self, _id: u64, _key: &str) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn mark_done(&self, _id: u64) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn mark_abandoned(&self, _id: u64) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn mark_mistaken(&self, _id: u64) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn mark_duplicated(&self, _id: u64) -> anyhow::Result<Task> {
            unreachable!("not used in web tests")
        }

        fn next_task(&self) -> anyhow::Result<Option<Task>> {
            Ok(self.tasks.first().cloned())
        }
    }

    #[tokio::test]
    async fn api_tasks_returns_task_json() {
        let backend = MockBackend {
            tasks: vec![Task {
                id: Some(3),
                uuid: "abc".into(),
                core: CoreTaskFields {
                    title: "Ship MVP".into(),
                    status: TaskStatus::Unstarted,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    target_date: None,
                    deadline: None,
                    launch_date: None,
                    target_time_hint: None,
                    deadline_time_hint: None,
                    launch_time_hint: None,
                    project: None,
                    tags: Vec::new(),
                },
                annotations: Vec::new(),
                extra: Map::from_iter([
                    ("urgency".into(), Value::from(7.5)),
                    (
                        "chatwork".into(),
                        Value::Object(Map::from_iter([
                            ("requester".into(), Value::String("佐藤".into())),
                            ("abstract".into(), Value::String("Batch fix".into())),
                        ])),
                    ),
                ]),
            }],
        };
        let app = crate::web::app_router(backend);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tasks")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let text = String::from_utf8(body.to_vec()).expect("utf8");
        assert!(text.contains("\"title\":\"Ship MVP\""));
    }

    #[tokio::test]
    async fn index_page_renders_taskforce_heading() {
        let app = crate::web::app_router(MockBackend { tasks: Vec::new() });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let text = String::from_utf8(body.to_vec()).expect("utf8");
        assert!(text.contains("taskforce"));
        assert!(text.contains("Open Tasks"));
    }

    #[tokio::test]
    async fn detail_page_and_task_api_render_structured_task_data() {
        let backend = MockBackend {
            tasks: vec![Task {
                id: Some(7),
                uuid: "detail".into(),
                core: CoreTaskFields {
                    title: "Review scope output".into(),
                    status: TaskStatus::Active,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    target_date: None,
                    deadline: None,
                    launch_date: None,
                    target_time_hint: None,
                    deadline_time_hint: None,
                    launch_time_hint: None,
                    project: Some("taskforce".into()),
                    tags: vec!["chatwork".into()],
                },
                annotations: Vec::new(),
                extra: Map::from_iter([(
                    "chatwork".into(),
                    Value::Object(Map::from_iter([
                        ("requester".into(), Value::String("佐藤".into())),
                        (
                            "description".into(),
                            Value::String("Show the full details.".into()),
                        ),
                    ])),
                )]),
            }],
        };
        let app = crate::web::app_router(backend);

        let api_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/tasks/7")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(api_response.status(), StatusCode::OK);
        let api_body = to_bytes(api_response.into_body(), usize::MAX)
            .await
            .expect("body");
        let api_text = String::from_utf8(api_body.to_vec()).expect("utf8");
        assert!(api_text.contains("\"title\":\"Review scope output\""));

        let page_response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks/7")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(page_response.status(), StatusCode::OK);
        let page_body = to_bytes(page_response.into_body(), usize::MAX)
            .await
            .expect("body");
        let page_text = String::from_utf8(page_body.to_vec()).expect("utf8");
        assert!(page_text.contains("Description"));
        assert!(page_text.contains("Back to Open Tasks"));
    }
}
