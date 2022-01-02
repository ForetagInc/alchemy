use serde::{ Serialize, Deserialize };

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDocumentPropertyValues
{
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#type: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub min_length: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_length: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#enum: Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub items: Option<SchemaDocumentPropertyArray>
}

#[derive(Default, Serialize, Deserialize)]
pub struct SchemaDocumentProperty
{
	pub name: String,
	pub properties: SchemaDocumentPropertyValues
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaDocumentPropertyArray
{
	pub r#type: String,
	pub maximum: usize
}

pub type SchemaRequiredTypes = Vec<String>;