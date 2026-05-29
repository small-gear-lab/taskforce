use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type TaskId = Option<u64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub uuid: String,
    pub core: CoreTaskFields,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreTaskFields {
    pub title: String,
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
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NewTaskInput {
    pub title: String,
    pub target_date: Option<NaiveDate>,
    pub deadline: Option<NaiveDate>,
    pub launch_date: Option<NaiveDate>,
    pub target_time_hint: Option<String>,
    pub deadline_time_hint: Option<String>,
    pub launch_time_hint: Option<String>,
    pub project: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateTaskInput {
    pub title: Option<String>,
    pub target_date: Option<NaiveDate>,
    pub clear_target_date: bool,
    pub deadline: Option<NaiveDate>,
    pub clear_deadline: bool,
    pub launch_date: Option<NaiveDate>,
    pub clear_launch_date: bool,
    pub target_time_hint: Option<String>,
    pub clear_target_time_hint: bool,
    pub deadline_time_hint: Option<String>,
    pub clear_deadline_time_hint: bool,
    pub launch_time_hint: Option<String>,
    pub clear_launch_time_hint: bool,
    pub project: Option<String>,
    pub clear_project: bool,
    pub tags: Option<Vec<String>>,
    pub clear_tags: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Unstarted,
    Active,
    Pending,
    Done,
    Abandoned,
    Mistaken,
    Duplicated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub created_at: DateTime<Utc>,
    pub kind: AnnotationKind,
    pub body: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationKind {
    #[default]
    Note,
    Progress,
    Decision,
    Handover,
}

impl Task {
    pub fn new(id: TaskId, uuid: String, title: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            uuid,
            core: CoreTaskFields {
                title,
                status: TaskStatus::Unstarted,
                created_at: now,
                updated_at: now,
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
            extra: Map::new(),
        }
    }

    pub fn id_text(&self) -> String {
        self.id
            .map(|id| id.to_string())
            .unwrap_or_else(|| self.uuid.clone())
    }

    pub fn title(&self) -> &str {
        &self.core.title
    }

    pub fn urgency(&self) -> f64 {
        self.extra
            .get("urgency")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    }
}

pub trait TaskBackend {
    fn list_pending(&self) -> Result<Vec<Task>>;
    fn add(&self, input: NewTaskInput) -> Result<Task>;
    fn edit(&self, id: u64, input: UpdateTaskInput) -> Result<Task>;
    fn get_task(&self, id: u64) -> Result<Task>;
    fn set_extra(&self, id: u64, key: &str, value: Value) -> Result<Task>;
    fn get_extra(&self, id: u64, key: &str) -> Result<Option<Value>>;
    fn unset_extra(&self, id: u64, key: &str) -> Result<Task>;
    fn mark_done(&self, id: u64) -> Result<Task>;
    fn mark_abandoned(&self, id: u64) -> Result<Task>;
    fn mark_mistaken(&self, id: u64) -> Result<Task>;
    fn mark_duplicated(&self, id: u64) -> Result<Task>;
    fn next_task(&self) -> Result<Option<Task>>;
}
