// Copyright (c) 2026- Masaki Ishii
// Copyright (c) 2026- Small Gear Lab
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::Serialize;
use serde_json::{Map, Value};
use unicode_segmentation::UnicodeSegmentation;

use crate::backend::{Annotation, CoreTaskFields, Task};

const DESCRIPTION_PREVIEW_GRAPHEMES: usize = 72;

#[derive(Debug, Clone, Serialize)]
pub struct TaskDto {
    pub id: Option<u64>,
    pub uuid: String,
    pub core: CoreTaskFieldsDto,
    pub annotations: Vec<AnnotationDto>,
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskListItemDto {
    pub id: Option<u64>,
    pub uuid: String,
    pub title: String,
    pub status: String,
    pub tags: Vec<String>,
    pub deadline: Option<String>,
    pub target_date: Option<String>,
    pub launch_date: Option<String>,
    pub description_preview: Option<String>,
    pub urgency: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoreTaskFieldsDto {
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub target_date: Option<String>,
    pub deadline: Option<String>,
    pub launch_date: Option<String>,
    pub target_time_hint: Option<String>,
    pub deadline_time_hint: Option<String>,
    pub launch_time_hint: Option<String>,
    pub project: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnotationDto {
    pub created_at: String,
    pub kind: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl From<&Task> for TaskDto {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id,
            uuid: task.uuid.clone(),
            core: CoreTaskFieldsDto::from(&task.core),
            annotations: task.annotations.iter().map(AnnotationDto::from).collect(),
            extra: canonicalize_object(&task.extra),
        }
    }
}

impl From<&Task> for TaskListItemDto {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id,
            uuid: task.uuid.clone(),
            title: task.core.title.clone(),
            status: task.core.status.to_string(),
            tags: task.core.tags.clone(),
            deadline: task.core.deadline.map(|value| value.to_string()),
            target_date: task.core.target_date.map(|value| value.to_string()),
            launch_date: task.core.launch_date.map(|value| value.to_string()),
            description_preview: task
                .core
                .description
                .as_deref()
                .map(|text| truncate_graphemes(text, DESCRIPTION_PREVIEW_GRAPHEMES)),
            urgency: task.urgency(),
        }
    }
}

impl From<&CoreTaskFields> for CoreTaskFieldsDto {
    fn from(core: &CoreTaskFields) -> Self {
        Self {
            title: core.title.clone(),
            description: core.description.clone(),
            status: core.status.to_string(),
            created_at: core.created_at.to_rfc3339(),
            updated_at: core.updated_at.to_rfc3339(),
            target_date: core.target_date.map(|value| value.to_string()),
            deadline: core.deadline.map(|value| value.to_string()),
            launch_date: core.launch_date.map(|value| value.to_string()),
            target_time_hint: core.target_time_hint.clone(),
            deadline_time_hint: core.deadline_time_hint.clone(),
            launch_time_hint: core.launch_time_hint.clone(),
            project: core.project.clone(),
            tags: core.tags.clone(),
        }
    }
}

impl From<&Annotation> for AnnotationDto {
    fn from(annotation: &Annotation) -> Self {
        Self {
            created_at: annotation.created_at.to_rfc3339(),
            kind: annotation.kind.to_string(),
            body: annotation.body.clone(),
            idempotency_key: annotation.idempotency_key.clone(),
        }
    }
}

fn canonicalize_object(map: &Map<String, Value>) -> Value {
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(left, _)| *left);

    let mut canonical = Map::new();
    for (key, value) in entries {
        canonical.insert(key.clone(), canonicalize_value(value));
    }

    Value::Object(canonical)
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        Value::Object(map) => canonicalize_object(map),
        other => other.clone(),
    }
}

fn truncate_graphemes(text: &str, max_graphemes: usize) -> String {
    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() <= max_graphemes {
        return text.to_string();
    }

    format!("{}…", graphemes[..max_graphemes].concat())
}

#[cfg(test)]
mod tests {
    use super::truncate_graphemes;

    #[test]
    fn truncates_by_grapheme_cluster() {
        let text = "👨‍👩‍👧‍👦abc";
        assert_eq!(truncate_graphemes(text, 1), "👨‍👩‍👧‍👦…");
        assert_eq!(truncate_graphemes(text, 4), text);
    }
}
