const configNode = document.getElementById("taskforce-index-config");
const config = configNode ? JSON.parse(configNode.textContent ?? "{}") : {};
const labels = config.labels ?? {};
const statusLabels = config.status_labels ?? {};

const taskList = document.getElementById("task-list");
const emptyState = document.getElementById("empty");
const refreshButton = document.getElementById("refresh");
const statusOrder = ["active", "unstarted", "waiting", "suspended"];
const apiUrl = config.api_url ?? "/api/tasks";

function label(name, fallback) {
  return labels[name] ?? fallback;
}

function statusLabel(status) {
  return statusLabels[status] ?? status ?? "unknown";
}

function formatDate(value) {
  return value ?? null;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;")
    .replaceAll("'", "&#39;");
}

function createTaskItem(task) {
  const item = document.createElement("li");
  const deadline = formatDate(task.deadline);
  const target = formatDate(task.target_date);
  const launch = formatDate(task.launch_date);
  const descriptionPreview = task.description_preview;
  const hashtags = Array.isArray(task.tags)
    ? task.tags
        .map(
          (tag) =>
            `<a class="task-hashtag" href="/tags/${encodeURIComponent(tag)}">#${escapeHtml(tag)}</a>`,
        )
        .join("")
    : "";
  item.innerHTML = `
    <span class="task-id">#${task.id ?? "?"}</span>
    <div class="task-main">
      <div class="task-title-row">
        <a class="task-link" href="/tasks/${task.id ?? ""}"></a>
        <span class="task-status task-status--${task.status}">${statusLabel(task.status)}</span>
      </div>
      ${descriptionPreview ? `<div class="task-description-preview">${escapeHtml(descriptionPreview)}</div>` : ""}
      ${hashtags ? `<div class="task-hashtags">${hashtags}</div>` : ""}
      <div class="task-meta">
        <span class="task-meta-item task-meta-item--deadline">${label("deadline", "Deadline")} ${deadline ?? label("no_deadline", "No deadline")}</span>
        ${target ? `<span class="task-meta-item">${label("target", "Target")} ${target}</span>` : ""}
        ${launch ? `<span class="task-meta-item">${label("launch", "Launch")} ${launch}</span>` : ""}
      </div>
    </div>
    <span class="task-urgency">${label("urgency", "urgency")} ${Number(task.urgency ?? 0).toFixed(1)}</span>
  `;
  item.querySelector(".task-link").textContent = task.title;
  return item;
}

function groupTasks(tasks) {
  const groups = new Map();
  for (const status of statusOrder) {
    groups.set(status, []);
  }

  for (const task of tasks) {
    const status = task.status;
    if (!groups.has(status)) {
      groups.set(status, []);
    }
    groups.get(status).push(task);
  }

  return groups;
}

async function loadTasks() {
  const response = await fetch(apiUrl);
  const tasks = await response.json();

  taskList.innerHTML = "";
  emptyState.hidden = tasks.length !== 0;
  emptyState.textContent = label("no_open_tasks", "No open tasks.");

  const groups = groupTasks(tasks);
  for (const [status, groupedTasks] of groups.entries()) {
    if (groupedTasks.length === 0) {
      continue;
    }

    const section = document.createElement("li");
    section.className = "task-group";

    const details = document.createElement("details");
    details.className = "task-group-details";
    details.open = status === "active";

    const summary = document.createElement("summary");
    summary.className = "task-group-summary";
    summary.innerHTML = `
      <span class="task-group-leading">
        <span class="task-group-chevron" aria-hidden="true">▾</span>
      </span>
      <span class="task-group-heading">
        <span class="task-status task-status--${status}">${statusLabel(status)}</span>
        <span class="task-group-count">${groupedTasks.length} ${label("tasks", "tasks")}</span>
      </span>
    `;

    const innerList = document.createElement("ul");
    innerList.className = "task-group-list";
    for (const task of groupedTasks) {
      innerList.appendChild(createTaskItem(task));
    }

    details.append(summary, innerList);
    section.appendChild(details);
    taskList.appendChild(section);
  }
}

refreshButton.addEventListener("click", () => {
  loadTasks().catch(console.error);
});

loadTasks().catch(console.error);
