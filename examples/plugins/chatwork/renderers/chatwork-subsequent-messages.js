import {
  normalizeBlocks,
  parseRenderBlocks,
  renderRequestBlock,
} from "./chatwork-render-blocks.js";

function formatSentAt(value) {
  if (typeof value !== "string" || value.trim() === "") {
    return "";
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  const yyyy = parsed.getFullYear();
  const mm = String(parsed.getMonth() + 1).padStart(2, "0");
  const dd = String(parsed.getDate()).padStart(2, "0");
  const hh = String(parsed.getHours()).padStart(2, "0");
  const mi = String(parsed.getMinutes()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd} ${hh}:${mi}`;
}

function entryBlocks(entry, context) {
  if (Array.isArray(entry.render_blocks) && entry.render_blocks.length > 0) {
    return entry.render_blocks;
  }
  if (typeof entry.body_raw === "string" && entry.body_raw.trim() !== "") {
    return parseRenderBlocks(entry.body_raw, null).blocks;
  }
  return normalizeBlocks(null, context);
}

function renderEntry(entry, helpers, context) {
  const card = document.createElement("article");
  card.className = "request-block request-block--info subsequent-message";

  const header = document.createElement("div");
  header.className = "subsequent-message__header";

  const senderName = entry?.sender?.name ?? "";
  if (senderName) {
    const sender = document.createElement("span");
    sender.className = "subsequent-message__sender";
    sender.textContent = senderName;
    header.appendChild(sender);
  }

  const sentAt = formatSentAt(entry?.sent_at);
  if (sentAt) {
    const time = document.createElement("time");
    time.className = "subsequent-message__sent-at";
    if (typeof entry?.sent_at === "string") {
      time.setAttribute("datetime", entry.sent_at);
    }
    time.textContent = sentAt;
    header.appendChild(time);
  }

  if (typeof entry?.url === "string" && entry.url.trim() !== "" && helpers?.createExternalLink) {
    const link = helpers.createExternalLink(entry.url);
    link.classList.add("subsequent-message__link");
    link.textContent = context.message("open_in_chatwork", "open");
    header.appendChild(link);
  }

  card.appendChild(header);

  const body = document.createElement("div");
  body.className = "subsequent-message__body section-stack";
  const blocks = entryBlocks(entry ?? {}, context);
  for (const block of blocks) {
    body.appendChild(renderRequestBlock(block, helpers));
  }
  card.appendChild(body);

  return card;
}

function pickEntries(value, context) {
  if (Array.isArray(value) && value.length > 0) {
    return value;
  }
  const fromTask = context.task?.extra?.[context.pluginId]?.subsequent_messages;
  if (Array.isArray(fromTask)) {
    return fromTask;
  }
  return [];
}

export function render(value, context) {
  const helpers = context.helpers ?? {};
  const root = document.createElement("div");

  const entries = pickEntries(value, context);
  if (entries.length === 0) {
    root.className = "section-body empty";
    root.textContent = context.message(
      "no_subsequent_messages",
      "No subsequent messages yet."
    );
    return root;
  }

  root.className = "section-stack subsequent-messages";
  for (const entry of entries) {
    root.appendChild(renderEntry(entry ?? {}, helpers, context));
  }
  return root;
}
