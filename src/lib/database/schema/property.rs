use serde::{ Serialize, Deserialize };

use super::SchemaNativeTypeArray;

/// The schema property
#[derive(Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchemaProperty
{
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#type: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub min_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#enum: Option<Vec<String>>,
	/// Only set if the type is an Array
	#[serde(skip_serializing_if = "Option::is_none")]
	pub items: Option<SchemaNativeTypeArray>
}

impl SchemaProperty
{
	pub fn new() -> Self
	{
		Default::default()
	}
}