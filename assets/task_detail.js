const configNode = document.getElementById("taskforce-detail-config");
const config = configNode ? JSON.parse(configNode.textContent ?? "{}") : {};
const labels = config.labels ?? {};
const messages = config.messages ?? {};
const statusLabels = config.status_labels ?? {};
const logicalLabels = config.logical_labels ?? {};

const taskId = window.location.pathname.split("/").pop();
const legacyChatworkKeys = [
  "requester",
  "request_url",
  "related_request_url",
  "summary",
  "abstract",
  "description",
  "production_rollout",
  "template_kind",
  "target_sites",
  "render_blocks",
  "source",
];

function label(name, fallback) {
  return labels[name] ?? fallback;
}

function message(name, fallback) {
  return messages[name] ?? fallback;
}

function textOrFallback(value, fallback = label("dash", "—")) {
  return value == null || value === "" ? fallback : value;
}

function isExternalUrl(value) {
  return typeof value === "string" && /^https?:\/\/\S+$/i.test(value.trim());
}

function isUrlFieldPath(path) {
  return (
    path.endsWith(".url") ||
    path.endsWith("_url") ||
    path === "request_url" ||
    path === "related_request_url" ||
    path === "chatwork.request_url" ||
    path === "chatwork.related_request_url"
  );
}

function createExternalLink(url, labelText = url) {
  const link = document.createElement("a");
  link.href = url;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  link.textContent = labelText;
  link.className = "external-link";
  return link;
}

function appendLinkifiedText(container, text) {
  const source = text ?? "";
  const urlPattern = /https?:\/\/\S+/gi;
  let lastIndex = 0;

  for (const match of source.matchAll(urlPattern)) {
    const url = match[0];
    const index = match.index ?? 0;
    if (index > lastIndex) {
      container.append(document.createTextNode(source.slice(lastIndex, index)));
    }
    container.append(createExternalLink(url));
    lastIndex = index + url.length;
  }

  if (lastIndex < source.length) {
    container.append(document.createTextNode(source.slice(lastIndex)));
  }
}

function dateLine(date, hint) {
  if (!date && !hint) return label("dash", "—");
  return [date, hint].filter(Boolean).join(" ");
}

function extractInfoBlock(text) {
  if (!text) {
    return null;
  }

  const match = text.match(/\[info\]([\s\S]*?)\[\/info\]/i);
  if (!match) {
    return text.trim() || null;
  }

  return match[1]
    .replace(/\[info\]|\[\/info\]/gi, "")
    .trim();
}

function parseChatworkRenderBlocks(text) {
  return parseRenderBlocks(text ?? "", null).blocks;
}

function pushTextRenderBlock(blocks, text) {
  const normalized = text.trim();
  if (!normalized) {
    return;
  }

  blocks.push({
    kind: "text",
    text: normalized,
  });
}

function parseInfoRenderBlock(text) {
  return parseInfoBlock(text).block;
}

function parseRenderBlocks(text, terminator) {
  const blocks = [];
  let index = 0;

  while (index < text.length) {
    const rest = text.slice(index);

    if (terminator && rest.startsWith(terminator)) {
      return { blocks, index };
    }

    if (rest.startsWith("[info]")) {
      const parsed = parseInfoBlock(rest);
      blocks.push(parsed.block);
      index += parsed.consumed;
      continue;
    }

    if (rest.startsWith("[code]")) {
      const parsed = parseCodeBlock(rest);
      blocks.push(parsed.block);
      index += parsed.consumed;
      continue;
    }

    if (rest.startsWith("[qt]")) {
      const parsed = parseQuoteBlock(rest);
      blocks.push(parsed.block);
      index += parsed.consumed;
      continue;
    }

    if (rest.startsWith("[hr]")) {
      blocks.push({
        kind: "rule",
        title: null,
        text: "",
        children: [],
      });
      index += "[hr]".length;
      continue;
    }

    const nextIndex = findNextMarkup(rest, terminator);
    pushTextRenderBlock(blocks, rest.slice(0, nextIndex));
    index += nextIndex;
  }

  return { blocks, index };
}

function parseInfoBlock(text) {
  let index = "[info]".length;
  while (index < text.length && /\s/.test(text[index])) {
    index += 1;
  }

  let title = null;
  if (text.slice(index).startsWith("[title]")) {
    const closeIndex = text.indexOf("[/title]", index + "[title]".length);
    if (closeIndex >= 0) {
      const titleStart = index + "[title]".length;
      title = text.slice(titleStart, closeIndex).trim();
      index = closeIndex + "[/title]".length;
    }
  }

  const parsed = parseRenderBlocks(text.slice(index), "[/info]");
  const children = [...parsed.blocks];
  let bodyText = "";
  if (children[0]?.kind === "text") {
    bodyText = children.shift().text ?? "";
  }

  const closeOffset = index + parsed.index;
  const consumed = text.slice(closeOffset).startsWith("[/info]")
    ? closeOffset + "[/info]".length
    : text.length;

  return {
    block: {
      kind: "info",
      title,
      text: bodyText,
      children,
    },
    consumed,
  };
}

function parseCodeBlock(text) {
  const closeIndex = text.indexOf("[/code]", "[code]".length);
  if (closeIndex >= 0) {
    return {
      block: {
        kind: "code",
        title: null,
        text: text.slice("[code]".length, closeIndex).trim(),
        children: [],
      },
      consumed: closeIndex + "[/code]".length,
    };
  }

  return {
    block: {
      kind: "code",
      title: null,
      text: text.trim(),
      children: [],
    },
    consumed: text.length,
  };
}

function parseQuoteBlock(text) {
  const innerStart = "[qt]".length;
  const parsed = parseRenderBlocks(text.slice(innerStart), "[/qt]");
  const children = [...parsed.blocks];
  let bodyText = "";
  if (children[0]?.kind === "text") {
    bodyText = children.shift().text ?? "";
  }

  const closeOffset = innerStart + parsed.index;
  const consumed = text.slice(closeOffset).startsWith("[/qt]")
    ? closeOffset + "[/qt]".length
    : text.length;

  return {
    block: {
      kind: "quote",
      title: null,
      text: bodyText,
      children,
    },
    consumed,
  };
}

function findNextMarkup(text, terminator) {
  const indexes = [];
  const infoIndex = text.indexOf("[info]");
  if (infoIndex >= 0) {
    indexes.push(infoIndex);
  }
  const codeIndex = text.indexOf("[code]");
  if (codeIndex >= 0) {
    indexes.push(codeIndex);
  }
  const quoteIndex = text.indexOf("[qt]");
  if (quoteIndex >= 0) {
    indexes.push(quoteIndex);
  }
  const ruleIndex = text.indexOf("[hr]");
  if (ruleIndex >= 0) {
    indexes.push(ruleIndex);
  }
  if (terminator) {
    const terminatorIndex = text.indexOf(terminator);
    if (terminatorIndex >= 0) {
      indexes.push(terminatorIndex);
    }
  }

  return indexes.length === 0 ? text.length : Math.min(...indexes);
}

function labelFor(path, fallbackKey) {
  return logicalLabels[path] ?? fallbackKey;
}

function isObject(value) {
  return value != null && typeof value === "object" && !Array.isArray(value);
}

function normalizeChatworkExtra(extra) {
  if (!isObject(extra)) {
    return null;
  }

  if (isObject(extra.chatwork)) {
    return extra.chatwork;
  }

  const chatwork = {};
  let hasLegacyChatworkData = false;
  for (const key of legacyChatworkKeys) {
    if (Object.hasOwn(extra, key)) {
      chatwork[key] = extra[key];
      hasLegacyChatworkData = true;
    }
  }

  if (!hasLegacyChatworkData) {
    return null;
  }

  return chatwork;
}

function filterPluginFields(pluginKey, pluginValue) {
  if (pluginKey === "chatwork" && isObject(pluginValue)) {
    const filtered = { ...pluginValue };
    delete filtered.source;
    delete filtered.render_blocks;
    return filtered;
  }

  return pluginValue;
}

function normalizePluginExtra(extra) {
  if (!isObject(extra)) {
    return {};
  }

  const namespaces = {};
  const chatwork = normalizeChatworkExtra(extra);
  if (chatwork && Object.keys(chatwork).length > 0) {
    namespaces.chatwork = filterPluginFields("chatwork", chatwork);
  }

  for (const [key, value] of Object.entries(extra)) {
    if (key === "chatwork" || legacyChatworkKeys.includes(key)) {
      continue;
    }
    namespaces[key] = filterPluginFields(key, value);
  }

  return namespaces;
}

function previewValue(value) {
  if (Array.isArray(value)) {
    return `${value.length} items`;
  }
  if (value && typeof value === "object") {
    return `${Object.keys(value).length} fields`;
  }
  if (value == null) {
    return label("dash", "—");
  }
  const text = String(value);
  return text.length > 52 ? `${text.slice(0, 52)}…` : text;
}

function renderJsonTree(path, key, value) {
  if (Array.isArray(value)) {
    if (key === "extra") {
      const wrapper = document.createElement("div");
      wrapper.className = "tree-root";
      value.forEach((item, index) => {
        wrapper.appendChild(renderJsonTree(`${path}[]`, `[${index}]`, item));
      });
      if (value.length === 0) {
        const empty = document.createElement("div");
        empty.className = "empty";
        empty.textContent = message("no_extra_data", "No extra data.");
        wrapper.appendChild(empty);
      }
      return wrapper;
    }

    const details = document.createElement("details");
    details.open = false;
    const summary = document.createElement("summary");
    summary.innerHTML = `
      <span class="tree-key">${labelFor(path, key)}</span>
      <span class="tree-preview">${previewValue(value)}</span>
    `;
    details.appendChild(summary);

    const content = document.createElement("div");
    content.className = "tree-content tree-children";
    if (value.length === 0) {
      const empty = document.createElement("div");
      empty.className = "empty";
      empty.textContent = message("no_extra_data", "No extra data.");
      content.appendChild(empty);
    } else {
      value.forEach((item, index) => {
        content.appendChild(renderJsonTree(`${path}[]`, `[${index}]`, item));
      });
    }
    details.appendChild(content);

    const wrapper = document.createElement("div");
    wrapper.className = "tree-node tree-node--branch";
    wrapper.appendChild(details);
    return wrapper;
  }

  if (value && typeof value === "object") {
    if (key === "extra") {
      const wrapper = document.createElement("div");
      wrapper.className = "tree-root";
      const entries = Object.entries(value);
      if (entries.length === 0) {
        const empty = document.createElement("div");
        empty.className = "empty";
        empty.textContent = message("no_extra_data", "No extra data.");
        wrapper.appendChild(empty);
      } else {
        for (const [childKey, childValue] of entries) {
          wrapper.appendChild(renderJsonTree(childKey, childKey, childValue));
        }
      }
      return wrapper;
    }

    const details = document.createElement("details");
    details.open = key === "chatwork";
    const summary = document.createElement("summary");
    summary.innerHTML = `
      <span class="tree-key">${labelFor(path, key)}</span>
      <span class="tree-preview">${previewValue(value)}</span>
    `;
    details.appendChild(summary);

    const content = document.createElement("div");
    content.className = "tree-content tree-children";
    const entries = Object.entries(value);
    if (entries.length === 0) {
      const empty = document.createElement("div");
      empty.className = "empty";
      empty.textContent = message("no_extra_data", "No extra data.");
      content.appendChild(empty);
    } else {
      for (const [childKey, childValue] of entries) {
        content.appendChild(renderJsonTree(`${path}.${childKey}`, childKey, childValue));
      }
    }
    details.appendChild(content);

    const wrapper = document.createElement("div");
    wrapper.className = "tree-node tree-node--branch";
    wrapper.appendChild(details);
    return wrapper;
  }

  const leaf = document.createElement("div");
  leaf.className = "tree-node tree-node--leaf tree-leaf";
  const keyNode = document.createElement("div");
  keyNode.className = "tree-key";
  keyNode.textContent = labelFor(path, key);
  leaf.appendChild(keyNode);

  const valueNode = document.createElement("div");
  valueNode.className = "tree-leaf-value";
  const textValue = textOrFallback(value);
  if (isUrlFieldPath(path) && isExternalUrl(textValue)) {
    valueNode.appendChild(createExternalLink(textValue));
  } else {
    valueNode.textContent = textValue;
  }
  leaf.appendChild(valueNode);
  return leaf;
}

function wireExtraViewToggle(treeButton, rawButton, expandAllButton, collapseAllButton, treeView, rawView) {
  function setMode(mode) {
    const treeActive = mode === "tree";
    treeButton.classList.toggle("is-active", treeActive);
    rawButton.classList.toggle("is-active", !treeActive);
    treeView.hidden = !treeActive;
    rawView.hidden = treeActive;
  }

  treeButton.addEventListener("click", () => setMode("tree"));
  rawButton.addEventListener("click", () => setMode("raw"));
  expandAllButton.addEventListener("click", () => {
    treeView.querySelectorAll("details").forEach((node) => {
      node.open = true;
    });
    setMode("tree");
  });
  collapseAllButton.addEventListener("click", () => {
    treeView.querySelectorAll("details").forEach((node) => {
      node.open = false;
    });
    setMode("tree");
  });
  setMode("tree");
}

function renderPluginExtraSection(pluginKey, pluginValue) {
  const section = document.createElement("div");
  section.className = "plugin-section";

  const details = document.createElement("details");
  details.open = true;

  const summary = document.createElement("summary");
  summary.className = "plugin-section__summary";
  summary.innerHTML = `<span class="kv-key">${labelFor(pluginKey, pluginKey)}</span>`;
  details.appendChild(summary);

  const content = document.createElement("div");
  content.className = "plugin-section__content";

  const tools = document.createElement("div");
  tools.className = "extra-tools";
  tools.innerHTML = `
    <div class="view-toggle" role="tablist" aria-label="${pluginKey}-data-view">
      <button class="is-active" type="button">${label("tree", "Tree")}</button>
      <button type="button">${label("raw", "Raw")}</button>
    </div>
    <button type="button">${label("expand_all", "Expand all")}</button>
    <button type="button">${label("collapse_all", "Collapse all")}</button>
  `;
  content.appendChild(tools);

  const treeView = document.createElement("div");
  treeView.className = "extra-view tree-root";
  if (pluginValue && typeof pluginValue === "object" && !Array.isArray(pluginValue)) {
    const entries = Object.entries(pluginValue);
    if (entries.length === 0) {
      const empty = document.createElement("div");
      empty.className = "empty";
      empty.textContent = message("no_extra_data", "No extra data.");
      treeView.appendChild(empty);
    } else {
      for (const [childKey, childValue] of entries) {
        treeView.appendChild(
          renderJsonTree(`${pluginKey}.${childKey}`, childKey, childValue)
        );
      }
    }
  } else {
    treeView.appendChild(renderJsonTree(pluginKey, pluginKey, pluginValue));
  }
  content.appendChild(treeView);

  const rawView = document.createElement("pre");
  rawView.className = "extra-view";
  rawView.hidden = true;
  rawView.textContent = JSON.stringify(pluginValue, null, 2);
  content.appendChild(rawView);

  details.appendChild(content);
  section.appendChild(details);

  wireExtraViewToggle(
    tools.querySelector(".view-toggle button:first-child"),
    tools.querySelector(".view-toggle button:last-child"),
    tools.querySelectorAll("button")[2],
    tools.querySelectorAll("button")[3],
    treeView,
    rawView
  );

  return section;
}

function renderPluginExtraSections(container, extra) {
  const namespaces = Object.entries(normalizePluginExtra(extra));

  if (namespaces.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent = message("no_extra_data", "No extra data.");
    container.appendChild(empty);
    return;
  }

  for (const [pluginKey, pluginValue] of namespaces) {
    container.appendChild(renderPluginExtraSection(pluginKey, pluginValue));
  }
}

function normalizeOriginalRequestBlocks(chatwork) {
  if (Array.isArray(chatwork.render_blocks) && chatwork.render_blocks.length > 0) {
    return chatwork.render_blocks;
  }

  const source = chatwork.source ?? {};
  return parseChatworkRenderBlocks(source.body_raw);
}

function renderOriginalRequest(container, chatwork) {
  const blocks = normalizeOriginalRequestBlocks(chatwork);
  container.innerHTML = "";

  if (blocks.length === 0) {
    container.textContent = message(
      "no_original_request",
      "Original request text is not available."
    );
    container.className = "section-body empty";
    return;
  }

  container.className = "section-stack";

  for (const block of blocks) {
    container.appendChild(renderRequestBlock(block));
  }
}

function renderRequestBlock(block) {
  if (block.kind === "rule") {
    const rule = document.createElement("hr");
    rule.className = "request-block request-block--rule";
    return rule;
  }

  if (block.kind === "quote") {
    const quote = document.createElement("blockquote");
    quote.className = "request-block request-block--quote";
    if (block.text) {
      appendLinkifiedText(quote, block.text ?? "");
    }
    if (Array.isArray(block.children) && block.children.length > 0) {
      const children = document.createElement("div");
      children.className = "request-block__children";
      for (const child of block.children) {
        children.appendChild(renderRequestBlock(child));
      }
      quote.appendChild(children);
    }
    return quote;
  }

  if (block.kind === "code") {
    const pre = document.createElement("pre");
    pre.className = "request-block request-block--code";
    pre.textContent = block.text ?? "";
    return pre;
  }

  if (block.kind === "info") {
    const wrapper = document.createElement("section");
    wrapper.className = "request-block request-block--info";
    if (block.title) {
      const title = document.createElement("div");
      title.className = "request-block__title";
      title.textContent = block.title;
      wrapper.appendChild(title);
    }
    if (block.text) {
      const body = document.createElement("div");
      body.className = "request-block__body";
      appendLinkifiedText(body, block.text);
      wrapper.appendChild(body);
    }
    if (Array.isArray(block.children) && block.children.length > 0) {
      const children = document.createElement("div");
      children.className = "request-block__children";
      for (const child of block.children) {
        children.appendChild(renderRequestBlock(child));
      }
      wrapper.appendChild(children);
    }
    return wrapper;
  }

  const paragraph = document.createElement("div");
  paragraph.className = "request-block request-block--text section-body";
  appendLinkifiedText(paragraph, block.text ?? "");
  return paragraph;
}

async function loadTask() {
  const response = await fetch(`/api/tasks/${taskId}`);
  if (!response.ok) {
    document.getElementById("task-title").textContent = message("task_not_found", "Task not found");
    document.getElementById("task-abstract").textContent = message(
      "task_could_not_be_loaded",
      "The requested task could not be loaded."
    );
    return;
  }

  const task = await response.json();
  const chatwork = normalizeChatworkExtra(task.extra) ?? {};
  const source = chatwork.source ?? {};
  const metaLine = document.getElementById("meta-line");
  const schedule = document.getElementById("schedule");
  const tagList = document.getElementById("tag-list");
  const pluginExtraSections = document.getElementById("plugin-extra-sections");

  document.title = `${task.core.title} | taskforce`;
  document.getElementById("task-title").textContent = task.core.title;
  document.getElementById("task-abstract").textContent = textOrFallback(
    chatwork.abstract || chatwork.summary,
    message("no_abstract_yet", "No abstract yet.")
  );
  document.getElementById("task-description").textContent = textOrFallback(
    chatwork.description,
    message("no_description_yet", "No description yet.")
  );
  renderOriginalRequest(document.getElementById("task-original-request"), chatwork);
  document.getElementById("project-value").textContent = textOrFallback(
    task.core.project,
    message("no_project", "no project")
  );
  pluginExtraSections.innerHTML = "";
  renderPluginExtraSections(pluginExtraSections, task.extra);

  metaLine.innerHTML = "";
  for (const chipText of [
    `#${task.id ?? "?"}`,
    statusLabels[task.core.status] ?? task.core.status,
    task.core.project ?? message("no_project", "no project"),
    `${task.annotations?.length ?? 0} ${label("annotations", "annotations")}`
  ]) {
    const chip = document.createElement("span");
    chip.className = "chip";
    chip.textContent = chipText;
    metaLine.appendChild(chip);
  }

  schedule.innerHTML = "";
  for (const [name, value] of [
    ["target", dateLine(task.core.target_date, task.core.target_time_hint)],
    ["deadline", dateLine(task.core.deadline, task.core.deadline_time_hint)],
    ["launch", dateLine(task.core.launch_date, task.core.launch_time_hint)]
  ]) {
    const row = document.createElement("div");
    row.className = "schedule-row";
    row.innerHTML = `
      <div class="schedule-label">${label(name, name)}</div>
      <div class="schedule-value">${value}</div>
    `;
    schedule.appendChild(row);
  }

  tagList.innerHTML = "";
  if ((task.core.tags ?? []).length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent = message("no_tags", "No tags.");
    tagList.appendChild(empty);
  } else {
    for (const tag of task.core.tags) {
      const item = document.createElement("span");
      item.className = "tag";
      item.textContent = tag;
      tagList.appendChild(item);
    }
  }
}

loadTask().catch(console.error);
