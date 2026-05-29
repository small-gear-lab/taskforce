use serde_json::{Map, Value};

pub type PluginId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalFieldLabel {
    pub physical_path: &'static str,
    pub msgid: &'static str,
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

pub fn logical_field_labels() -> &'static [LogicalFieldLabel] {
    crate::chatwork_plugin::logical_field_labels()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::PluginExtra;

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
}
