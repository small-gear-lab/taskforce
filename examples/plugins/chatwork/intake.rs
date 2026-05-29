use anyhow::{Result, anyhow};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde_json::{Value, json};

use crate::backend::{Annotation, NewTaskInput};
use crate::plugin::{PluginExtra, PluginId, RenderBlock};

const CHATWORK_PLUGIN_ID: PluginId = "chatwork";

#[derive(Debug, Clone)]
pub struct ChatworkMessage {
    pub room_id: u64,
    pub message_id: String,
    pub source_url: String,
    pub sender_account_id: u64,
    pub sender_name: String,
    pub body: String,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskDraft {
    pub input: NewTaskInput,
    pub extra: PluginExtra,
    #[allow(dead_code)]
    pub annotations: Vec<Annotation>,
}

pub trait TaskDraftPlugin {
    fn plugin_id(&self) -> PluginId;
    fn apply(&self, message: &ChatworkMessage, draft: &mut TaskDraft) -> Result<()>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChatworkSourcePlugin;

impl TaskDraftPlugin for ChatworkSourcePlugin {
    fn plugin_id(&self) -> PluginId {
        CHATWORK_PLUGIN_ID
    }

    fn apply(&self, message: &ChatworkMessage, draft: &mut TaskDraft) -> Result<()> {
        draft.extra.insert(
            self.plugin_id(),
            "source",
            json!({
                "kind": "chatwork",
                "room_id": message.room_id,
                "message_id": message.message_id,
                "url": message.source_url,
                "sender": {
                    "account_id": message.sender_account_id,
                    "name": message.sender_name,
                },
                "sent_at": message.sent_at.to_rfc3339(),
                "body_raw": message.body,
            }),
        );
        draft.extra.insert(
            self.plugin_id(),
            "render_blocks",
            Value::Array(
                parse_chatwork_render_blocks(&message.body)
                    .into_iter()
                    .map(RenderBlock::into_value)
                    .collect(),
            ),
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompanyRequestTemplatePlugin;

impl TaskDraftPlugin for CompanyRequestTemplatePlugin {
    fn plugin_id(&self) -> PluginId {
        CHATWORK_PLUGIN_ID
    }

    fn apply(&self, message: &ChatworkMessage, draft: &mut TaskDraft) -> Result<()> {
        let sections = parse_sections(&message.body);

        if draft.input.title.is_empty()
            && let Some(title) = extract_request_title(&message.body)
        {
            draft.input.title = title;
        }

        if let Some(lines) = sections.get("納期（日付を記載する）") {
            let joined = lines.join("\n");
            let (target_date, deadline) = parse_target_and_deadline(&joined, message.sent_at)?;
            draft.input.target_date = target_date;
            draft.input.deadline = deadline;
        }

        if let Some(lines) = sections.get("改修概要") {
            draft.extra.insert(
                self.plugin_id(),
                "summary",
                Value::String(lines.join("\n").trim().to_string()),
            );
        }

        if let Some(lines) = sections.get("依頼主")
            && let Some(requester) = first_nonempty_line(lines)
        {
            draft.extra.insert(
                self.plugin_id(),
                "requester",
                Value::String(requester.to_string()),
            );
        }

        if let Some(lines) = sections.get("過去の依頼(類似依頼)チャット")
            && let Some(url) = first_nonempty_line(lines)
        {
            draft.extra.insert(
                self.plugin_id(),
                "related_request_url",
                Value::String(url.to_string()),
            );
        }

        draft.extra.insert(
            self.plugin_id(),
            "request_url",
            Value::String(message.source_url.clone()),
        );

        if let Some(lines) = sections.get("対象サイト") {
            draft.extra.insert(
                self.plugin_id(),
                "target_sites",
                Value::Array(parse_target_sites(lines)),
            );
        }

        if let Some(lines) = sections.get("改修内容") {
            if draft.input.description.is_none() {
                draft.input.description = Some(lines.join("\n").trim().to_string());
            }
            draft.extra.insert(
                self.plugin_id(),
                "description",
                Value::String(lines.join("\n").trim().to_string()),
            );
        }

        if let Some(lines) = sections.get("改修目的") {
            draft.extra.insert(
                self.plugin_id(),
                "abstract",
                Value::String(lines.join("\n").trim().to_string()),
            );
        }

        if let Some(lines) = sections.get("本番反映") {
            draft.extra.insert(
                self.plugin_id(),
                "production_rollout",
                Value::String(lines.join("\n").trim().to_string()),
            );
        }

        draft.extra.insert(
            self.plugin_id(),
            "template_kind",
            Value::String("company_request".into()),
        );

        Ok(())
    }
}

pub fn build_draft_from_chatwork(message: &ChatworkMessage) -> Result<TaskDraft> {
    let mut draft = TaskDraft::default();
    ChatworkSourcePlugin.apply(message, &mut draft)?;
    CompanyRequestTemplatePlugin.apply(message, &mut draft)?;
    Ok(draft)
}

fn extract_request_title(body: &str) -> Option<String> {
    body.lines()
        .map(str::trim)
        .find(|line| line.starts_with("#/"))
        .map(|line| line.trim_start_matches('#').trim().to_string())
}

fn parse_sections(body: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut sections = std::collections::BTreeMap::new();
    let mut current: Option<String> = None;

    for raw_line in body.lines() {
        let line = raw_line.trim();
        if line.is_empty() || matches!(line, "[info]" | "[/info]" | "[code]" | "[/code]") {
            continue;
        }

        if let Some(name) = line.strip_prefix('■') {
            current = Some(name.to_string());
            sections.entry(name.to_string()).or_insert_with(Vec::new);
            continue;
        }

        if let Some(section) = current.as_ref() {
            sections
                .entry(section.clone())
                .or_insert_with(Vec::new)
                .push(line.to_string());
        }
    }

    sections
}

fn parse_target_and_deadline(
    text: &str,
    sent_at: DateTime<Utc>,
) -> Result<(Option<NaiveDate>, Option<NaiveDate>)> {
    let target = extract_jp_month_day_after(text, "希望：", sent_at)?;
    let deadline = extract_jp_month_day_after(text, "マスト：", sent_at)?;
    Ok((target, deadline))
}

fn extract_jp_month_day_after(
    text: &str,
    marker: &str,
    sent_at: DateTime<Utc>,
) -> Result<Option<NaiveDate>> {
    let Some(start) = text.find(marker) else {
        return Ok(None);
    };

    let tail = &text[start + marker.len()..];
    let month_idx = match tail.find('月') {
        Some(index) => index,
        None => return Ok(None),
    };
    let day_idx = match tail.find('日') {
        Some(index) => index,
        None => return Ok(None),
    };

    let month = tail[..month_idx]
        .trim()
        .parse::<u32>()
        .map_err(|err| anyhow!("invalid month near `{marker}`: {err}"))?;
    let day = tail[month_idx + "月".len()..day_idx]
        .trim()
        .parse::<u32>()
        .map_err(|err| anyhow!("invalid day near `{marker}`: {err}"))?;

    Ok(NaiveDate::from_ymd_opt(sent_at.year(), month, day))
}

fn first_nonempty_line(lines: &[String]) -> Option<&str> {
    lines
        .iter()
        .map(String::as_str)
        .find(|line| !line.is_empty())
}

fn parse_target_sites(lines: &[String]) -> Vec<Value> {
    lines
        .iter()
        .filter_map(|line| {
            let raw = line.trim();
            if raw.is_empty() {
                return None;
            }

            let site_code = raw
                .rfind('<')
                .zip(raw.rfind('>'))
                .and_then(|(start, end)| (start < end).then(|| raw[start + 1..end].trim()));

            let before_code = raw
                .rfind('<')
                .map(|index| raw[..index].trim_end())
                .unwrap_or(raw);
            let label = before_code
                .split(':')
                .next()
                .map(str::trim)
                .unwrap_or(before_code);

            Some(json!({
                "label": label,
                "site_code": site_code,
                "raw": raw,
            }))
        })
        .collect()
}

fn parse_chatwork_render_blocks(body: &str) -> Vec<RenderBlock> {
    parse_render_blocks(body, None).0
}

fn parse_render_blocks(text: &str, terminator: Option<&str>) -> (Vec<RenderBlock>, usize) {
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];

        if let Some(tag) = terminator
            && rest.starts_with(tag)
        {
            return (blocks, index);
        }

        if rest.starts_with("[info]") {
            let (block, consumed) = parse_info_block(rest);
            blocks.push(block);
            index += consumed;
            continue;
        }

        if rest.starts_with("[code]") {
            let (block, consumed) = parse_code_block(rest);
            blocks.push(block);
            index += consumed;
            continue;
        }

        if rest.starts_with("[qt]") {
            let (block, consumed) = parse_quote_block(rest);
            blocks.push(block);
            index += consumed;
            continue;
        }

        if rest.starts_with("[hr]") {
            blocks.push(RenderBlock::rule());
            index += "[hr]".len();
            continue;
        }

        let next_index = find_next_markup(rest, terminator);
        push_text_block(&mut blocks, &rest[..next_index]);
        index += next_index;
    }

    (blocks, index)
}

fn parse_info_block(text: &str) -> (RenderBlock, usize) {
    let mut index = "[info]".len();
    while let Some(ch) = text[index..].chars().next() {
        if ch.is_whitespace() {
            index += ch.len_utf8();
        } else {
            break;
        }
    }

    let mut title = None;
    if text[index..].starts_with("[title]")
        && let Some(end_index) = text[index + "[title]".len()..].find("[/title]")
    {
        let title_start = index + "[title]".len();
        let title_end = title_start + end_index;
        title = Some(text[title_start..title_end].trim().to_string());
        index = title_end + "[/title]".len();
    }

    let (mut children, inner_end) = parse_render_blocks(&text[index..], Some("[/info]"));
    let mut body_text = String::new();
    if matches!(children.first(), Some(first) if first.kind == crate::plugin::RenderBlockKind::Text)
    {
        let first = children.remove(0);
        body_text = first.text;
    }

    let close_offset = index + inner_end;
    let consumed = if text[close_offset..].starts_with("[/info]") {
        close_offset + "[/info]".len()
    } else {
        text.len()
    };

    (RenderBlock::info(title, body_text, children), consumed)
}

fn parse_code_block(text: &str) -> (RenderBlock, usize) {
    if let Some(end_index) = text["[code]".len()..].find("[/code]") {
        let code_start = "[code]".len();
        let code_end = code_start + end_index;
        let consumed = code_end + "[/code]".len();
        return (
            RenderBlock::code(text[code_start..code_end].trim()),
            consumed,
        );
    }

    (RenderBlock::code(text.trim()), text.len())
}

fn parse_quote_block(text: &str) -> (RenderBlock, usize) {
    let inner_start = "[qt]".len();
    let (mut children, inner_end) = parse_render_blocks(&text[inner_start..], Some("[/qt]"));
    let mut body_text = String::new();
    if matches!(children.first(), Some(first) if first.kind == crate::plugin::RenderBlockKind::Text)
    {
        let first = children.remove(0);
        body_text = first.text;
    }

    let close_offset = inner_start + inner_end;
    let consumed = if text[close_offset..].starts_with("[/qt]") {
        close_offset + "[/qt]".len()
    } else {
        text.len()
    };

    (RenderBlock::quote(body_text, children), consumed)
}

fn find_next_markup(text: &str, terminator: Option<&str>) -> usize {
    let mut indexes = Vec::new();
    if let Some(index) = text.find("[info]") {
        indexes.push(index);
    }
    if let Some(index) = text.find("[code]") {
        indexes.push(index);
    }
    if let Some(index) = text.find("[qt]") {
        indexes.push(index);
    }
    if let Some(index) = text.find("[hr]") {
        indexes.push(index);
    }
    if let Some(tag) = terminator
        && let Some(index) = text.find(tag)
    {
        indexes.push(index);
    }

    indexes.into_iter().min().unwrap_or(text.len())
}

fn push_text_block(blocks: &mut Vec<RenderBlock>, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    blocks.push(RenderBlock::text(trimmed));
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::Value;

    use super::{ChatworkMessage, build_draft_from_chatwork, parse_chatwork_render_blocks};

    const SAMPLE_BODY: &str = r#"[To:6023559] 石井将輝さん
#/batch/update_master.bash 正常に動作するようにする
#改修依頼
下記の改修をお願いできますでしょうか？
よろしくお願いします！

[info]
■納期（日付を記載する）
・希望：6月2日 マスト：6月5日
→希望は「ここまでだとありがたい」という期日で、マストは必須期日

■改修概要
・/batch/update_master.bash 正常に動作するようにする

■依頼主
佐藤

■過去の依頼(類似依頼)チャット
https://www.chatwork.com/#!rid36219958-1737709065104039936

■対象サイト
[info]
相鉄不動産販売さん       :219.94.162.164  (mansion9.sakura.ne.jp)<c-sotetsu>
リニュアル仲介さん       :59.106.27.193   (mansion37.sakura.ne.jp)<c-rchukai>
明和リアルエステートさん :153.126.170.17  (mansion64.mansion-library.com)<c-meiwa>
永大ハウス工業さん       :160.16.213.15   (mansion65.mansion-library.com)<c-eidaihouse>
さくらハウジングさん     :160.16.209.140  (mansion69.mansion-library.com)<c-sakura>
中央ベストホームさん     :153.126.130.48  (mansion76.mansion-library.com)<c-chuo>
エステート白馬さん       :160.16.196.209  (mansion77.mansion-library.com)<c-hakuba>
明和地所LPさん           :133.167.108.227 (mansion108.mansion-library.com)<c-meiwa-lp>
[/info]

■改修内容
上記サイトにて以下のようなエラーが出るようになったようです。

■改修目的
・/batch/update_master.bash 正常に動作するようにする

■本番反映
・即本番反映OK
[/info]"#;

    #[test]
    fn builds_task_draft_from_chatwork_message() {
        let message = ChatworkMessage {
            room_id: 36219958,
            message_id: "2111786210627420160".into(),
            source_url: "https://www.chatwork.com/#!rid36219958-2111786210627420160".into(),
            sender_account_id: 1343849,
            sender_name: "佐藤 幸二".into(),
            body: SAMPLE_BODY.into(),
            sent_at: Utc.with_ymd_and_hms(2026, 5, 28, 0, 0, 0).unwrap(),
        };

        let draft = build_draft_from_chatwork(&message).expect("draft");

        assert_eq!(
            draft.input.title,
            "/batch/update_master.bash 正常に動作するようにする"
        );
        assert_eq!(
            draft.input.target_date,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 2)
        );
        assert_eq!(
            draft.input.deadline,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 5)
        );
        assert_eq!(
            draft.extra.get("chatwork", "requester"),
            Some(&Value::String("佐藤".into()))
        );
        assert_eq!(
            draft.extra.get("chatwork", "template_kind"),
            Some(&Value::String("company_request".into()))
        );
        assert_eq!(
            draft
                .extra
                .get("chatwork", "target_sites")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(8)
        );
        assert_eq!(
            draft
                .extra
                .get("chatwork", "source")
                .and_then(Value::as_object)
                .and_then(|source| source.get("kind"))
                .and_then(Value::as_str),
            Some("chatwork")
        );
        assert_eq!(
            draft
                .extra
                .get("chatwork", "render_blocks")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        assert_eq!(
            draft
                .extra
                .get("chatwork", "render_blocks")
                .and_then(Value::as_array)
                .and_then(|blocks| blocks.get(1))
                .and_then(|block| block.get("children"))
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn parses_chatwork_render_blocks_for_info_and_code() {
        let body = r#"導入文です。

[info][title]改修依頼[/title]
詳細本文です。
[/info]

[code]
echo "hello"
[/code]
"#;

        let blocks = parse_chatwork_render_blocks(body);

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].kind.as_str(), "text");
        assert_eq!(blocks[0].text, "導入文です。");
        assert_eq!(blocks[1].kind.as_str(), "info");
        assert_eq!(blocks[1].title.as_deref(), Some("改修依頼"));
        assert_eq!(blocks[1].text, "詳細本文です。");
        assert!(blocks[1].children.is_empty());
        assert_eq!(blocks[2].kind.as_str(), "code");
        assert_eq!(blocks[2].text, "echo \"hello\"");
    }

    #[test]
    fn parses_nested_info_and_code_inside_info() {
        let body = r#"[info][title]親[/title]
前置きです。
[info][title]子[/title]
内側です。
[/info]
[code]
echo "[info]literal[/info]"
[/code]
[/info]"#;

        let blocks = parse_chatwork_render_blocks(body);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind.as_str(), "info");
        assert_eq!(blocks[0].title.as_deref(), Some("親"));
        assert_eq!(blocks[0].text, "前置きです。");
        assert_eq!(blocks[0].children.len(), 2);
        assert_eq!(blocks[0].children[0].kind.as_str(), "info");
        assert_eq!(blocks[0].children[0].title.as_deref(), Some("子"));
        assert_eq!(blocks[0].children[0].text, "内側です。");
        assert_eq!(blocks[0].children[1].kind.as_str(), "code");
        assert_eq!(blocks[0].children[1].text, "echo \"[info]literal[/info]\"");
    }

    #[test]
    fn parses_quote_and_rule_blocks() {
        let body = r#"導入文です。
[qt]
引用本文です。
[/qt]
[hr]
[code]
echo "hello"
[/code]
"#;

        let blocks = parse_chatwork_render_blocks(body);

        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].kind.as_str(), "text");
        assert_eq!(blocks[1].kind.as_str(), "quote");
        assert_eq!(blocks[1].text, "引用本文です。");
        assert!(blocks[1].children.is_empty());
        assert_eq!(blocks[2].kind.as_str(), "rule");
        assert_eq!(blocks[3].kind.as_str(), "code");
    }

    #[test]
    fn parses_quote_and_rule_inside_info() {
        let body = r#"[info][title]親[/title]
前置きです。
[qt]
引用本文です。
[/qt]
[hr]
[/info]"#;

        let blocks = parse_chatwork_render_blocks(body);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind.as_str(), "info");
        assert_eq!(blocks[0].text, "前置きです。");
        assert_eq!(blocks[0].children.len(), 2);
        assert_eq!(blocks[0].children[0].kind.as_str(), "quote");
        assert_eq!(blocks[0].children[0].text, "引用本文です。");
        assert_eq!(blocks[0].children[1].kind.as_str(), "rule");
    }

    #[test]
    fn parses_nested_blocks_inside_quote() {
        let body = r#"[qt]
前置きです。
[info][title]補足[/title]
引用内の情報です。
[/info]
[code]
echo "nested"
[/code]
[/qt]"#;

        let blocks = parse_chatwork_render_blocks(body);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind.as_str(), "quote");
        assert_eq!(blocks[0].text, "前置きです。");
        assert_eq!(blocks[0].children.len(), 2);
        assert_eq!(blocks[0].children[0].kind.as_str(), "info");
        assert_eq!(blocks[0].children[0].title.as_deref(), Some("補足"));
        assert_eq!(blocks[0].children[0].text, "引用内の情報です。");
        assert_eq!(blocks[0].children[1].kind.as_str(), "code");
        assert_eq!(blocks[0].children[1].text, "echo \"nested\"");
    }

    #[test]
    fn parses_nested_quote_inside_quote() {
        let body = r#"[qt]
外側の引用です。
[qt]
内側の引用です。
[info][title]内側メモ[/title]
内側の補足です。
[/info]
[code]
echo "deep"
[/code]
[/qt]
[/qt]"#;

        let blocks = parse_chatwork_render_blocks(body);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind.as_str(), "quote");
        assert_eq!(blocks[0].text, "外側の引用です。");
        assert_eq!(blocks[0].children.len(), 1);
        assert_eq!(blocks[0].children[0].kind.as_str(), "quote");
        assert_eq!(blocks[0].children[0].text, "内側の引用です。");
        assert_eq!(blocks[0].children[0].children.len(), 2);
        assert_eq!(blocks[0].children[0].children[0].kind.as_str(), "info");
        assert_eq!(
            blocks[0].children[0].children[0].title.as_deref(),
            Some("内側メモ")
        );
        assert_eq!(blocks[0].children[0].children[0].text, "内側の補足です。");
        assert_eq!(blocks[0].children[0].children[1].kind.as_str(), "code");
        assert_eq!(blocks[0].children[0].children[1].text, "echo \"deep\"");
    }
}
