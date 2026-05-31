const configNode = document.getElementById("taskforce-index-config");
const config = configNode ? JSON.parse(configNode.textContent ?? "{}") : {};
const labels = config.labels ?? {};
const statusLabels = config.status_labels ?? {};

const taskList = document.getElementById("task-list");
const emptyState = document.getElementById("empty");
const refreshButton = document.getElementById("refresh");

function label(name, fallback) {
  return labels[name] ?? fallback;
}

function statusLabel(status) {
  return statusLabels[status] ?? status ?? "unknown";
}

function formatDate(value) {
  return value ?? null;
}

async function loadTasks() {
  const response = await fetch("/api/tasks");
  const tasks = await response.json();

  taskList.innerHTML = "";
  emptyState.hidden = tasks.length !== 0;
  emptyState.textContent = label("no_open_tasks", "No open tasks.");

  for (const task of tasks) {
    const item = document.createElement("li");
    const deadline = formatDate(task.core.deadline);
    const target = formatDate(task.core.target_date);
    const launch = formatDate(task.core.launch_date);
    item.innerHTML = `
      <span class="task-id">#${task.id ?? "?"}</span>
      <div class="task-main">
        <div class="task-title-row">
          <a class="task-link" href="/tasks/${task.id ?? ""}"></a>
          <span class="task-status task-status--${task.core.status}">${statusLabel(task.core.status)}</span>
        </div>
        <div class="task-meta">
          <span class="task-meta-item task-meta-item--deadline">${label("deadline", "Deadline")} ${deadline ?? label("no_deadline", "No deadline")}</span>
          ${target ? `<span class="task-meta-item">${label("target", "Target")} ${target}</span>` : ""}
          ${launch ? `<span class="task-meta-item">${label("launch", "Launch")} ${launch}</span>` : ""}
        </div>
      </div>
      <span class="task-urgency">${label("urgency", "urgency")} ${Number(task.extra?.urgency ?? 0).toFixed(1)}</span>
    `;
    item.querySelector(".task-link").textContent = task.core.title;
    taskList.appendChild(item);
  }
}

refreshButton.addEventListener("click", () => {
  loadTasks().catch(console.error);
});

loadTasks().catch(console.error);
