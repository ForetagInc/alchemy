use serde::{ Serialize, Deserialize };

/// The Arango schema for a collection
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Rule
{
	pub r#type: String,
	pub properties: serde_json::Value,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub required: Option<Vec<String>>,
	pub additional_properties: bool
}