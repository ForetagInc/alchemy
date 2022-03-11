use serde::{Deserialize, Serialize};

/// The Arango schema for a collection
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub r#type: String,
    pub properties: serde_json::Value,
    pub required: Vec<String>,
    pub additional_properties: bool,
}
