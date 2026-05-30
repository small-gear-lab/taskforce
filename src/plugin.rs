use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

pub type PluginId = &'static str;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub custom_fields: Vec<PluginCustomField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginCustomField {
    pub path: String,
    pub label: String,
    #[serde(default)]
    pub placement: PluginFieldPlacement,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginFieldPlacement {
    Left,
    #[default]
    Right,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBlockKind {
    Text,
    Info,
    Code,
    Quote,
    Rule,
}

impl RenderBlockKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Info => "info",
            Self::Code => "code",
            Self::Quote => "quote",
            Self::Rule => "rule",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBlock {
    pub kind: RenderBlockKind,
    pub title: Option<String>,
    pub text: String,
    pub children: Vec<RenderBlock>,
}

impl RenderBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            kind: RenderBlockKind::Text,
            title: None,
            text: text.into(),
            children: Vec::new(),
        }
    }

    pub fn info(
        title: Option<String>,
        text: impl Into<String>,
        children: Vec<RenderBlock>,
    ) -> Self {
        Self {
            kind: RenderBlockKind::Info,
            title,
            text: text.into(),
            children,
        }
    }

    pub fn code(text: impl Into<String>) -> Self {
        Self {
            kind: RenderBlockKind::Code,
            title: None,
            text: text.into(),
            children: Vec::new(),
        }
    }

    pub fn quote(text: impl Into<String>, children: Vec<RenderBlock>) -> Self {
        Self {
            kind: RenderBlockKind::Quote,
            title: None,
            text: text.into(),
            children,
        }
    }

    pub fn rule() -> Self {
        Self {
            kind: RenderBlockKind::Rule,
            title: None,
            text: String::new(),
            children: Vec::new(),
        }
    }

    pub fn into_value(self) -> Value {
        json!({
            "kind": self.kind.as_str(),
            "title": self.title,
            "text": self.text,
            "children": self.children.into_iter().map(Self::into_value).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct PluginExtra {
    namespaces: Map<String, Value>,
}

impl PluginExtra {
    pub fn insert(&mut self, plugin_id: PluginId, key: impl Into<String>, value: Value) {
        self.namespace_mut(plugin_id).insert(key.into(), value);
    }

    pub fn get(&self, plugin_id: &str, key: &str) -> Option<&Value> {
        self.get_namespace(plugin_id)
            .and_then(|namespace| namespace.get(key))
    }

    pub fn get_namespace(&self, plugin_id: &str) -> Option<&Map<String, Value>> {
        self.namespaces.get(plugin_id).and_then(Value::as_object)
    }

    pub fn into_map(self) -> Map<String, Value> {
        self.namespaces
    }

    fn namespace_mut(&mut self, plugin_id: PluginId) -> &mut Map<String, Value> {
        let entry = self
            .namespaces
            .entry(plugin_id.to_string())
            .or_insert_with(|| Value::Object(Map::new()));

        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }

        entry
            .as_object_mut()
            .expect("plugin namespace should always be an object")
    }
}

pub fn plugin_manifests() -> Result<Vec<PluginManifest>> {
    Ok(vec![crate::chatwork_plugin::manifest()?])
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{PluginExtra, PluginFieldPlacement, RenderBlock, plugin_manifests};

    #[test]
    fn stores_values_under_plugin_namespaces() {
        let mut extra = PluginExtra::default();
        extra.insert("chatwork", "requester", Value::String("佐藤".into()));
        extra.insert(
            "chatwork",
            "template_kind",
            Value::String("company_request".into()),
        );

        assert_eq!(
            extra.get("chatwork", "requester"),
            Some(&Value::String("佐藤".into()))
        );
        assert_eq!(
            extra.get("chatwork", "template_kind"),
            Some(&Value::String("company_request".into()))
        );
        assert!(extra.get_namespace("missing").is_none());
    }

    #[test]
    fn render_block_serializes_into_expected_shape() {
        assert_eq!(
            RenderBlock::info(
                Some("改修依頼".into()),
                "詳細本文",
                vec![RenderBlock::code("echo hello")],
            )
            .into_value(),
            serde_json::json!({
                "kind": "info",
                "title": "改修依頼",
                "text": "詳細本文",
                "children": [{
                    "kind": "code",
                    "title": null,
                    "text": "echo hello",
                    "children": [],
                }],
            })
        );
        assert_eq!(
            RenderBlock::code("echo hello").into_value(),
            serde_json::json!({
                "kind": "code",
                "title": null,
                "text": "echo hello",
                "children": [],
            })
        );
        assert_eq!(
            RenderBlock::quote("quoted", vec![RenderBlock::code("echo nested")]).into_value(),
            serde_json::json!({
                "kind": "quote",
                "title": null,
                "text": "quoted",
                "children": [{
                    "kind": "code",
                    "title": null,
                    "text": "echo nested",
                    "children": [],
                }],
            })
        );
        assert_eq!(
            RenderBlock::rule().into_value(),
            serde_json::json!({
                "kind": "rule",
                "title": null,
                "text": "",
                "children": [],
            })
        );
    }

    #[test]
    fn loads_plugin_manifests() {
        let manifests = plugin_manifests().expect("plugin manifests");
        let chatwork = manifests
            .iter()
            .find(|manifest| manifest.id == "chatwork")
            .expect("chatwork manifest");

        assert_eq!(chatwork.name, "Chatwork");
        assert!(chatwork.custom_fields.iter().any(|field| {
            field.path == "render_blocks" && field.placement == PluginFieldPlacement::Left
        }));
        assert!(chatwork.custom_fields.iter().any(|field| {
            field.path == "source" && field.placement == PluginFieldPlacement::Hidden
        }));
    }
}
