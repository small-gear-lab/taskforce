const configNode = document.getElementById("taskforce-index-config");
const config = configNode ? JSON.parse(configNode.textContent ?? "{}") : {};
const labels = config.labels ?? {};

const taskList = document.getElementById("task-list");
const emptyState = document.getElementById("empty");
const refreshButton = document.getElementById("refresh");

function label(name, fallback) {
  return labels[name] ?? fallback;
}

async function loadTasks() {
  const response = await fetch("/api/tasks");
  const tasks = await response.json();

  taskList.innerHTML = "";
  emptyState.hidden = tasks.length !== 0;
  emptyState.textContent = label("no_open_tasks", "No open tasks.");

  for (const task of tasks) {
    const item = document.createElement("li");
    item.innerHTML = `
      <span class="task-id">#${task.id ?? "?"}</span>
      <span class="task-desc"><a class="task-link" href="/tasks/${task.id ?? ""}"></a></span>
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
