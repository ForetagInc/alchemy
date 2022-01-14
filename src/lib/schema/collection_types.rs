use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize)]
pub struct CollectionSchema
{
	pub message: String,
	pub level: String,
	pub rule: CollectionSchemaRule
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSchemaRule
{
	pub r#type: String,
	pub properties: serde_json::Value,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub required: Option<SchemaRequiredTypes>,
	pub additional_properties: bool
}

#[derive(Default, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDocumentPropertyValues
{
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#type: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub min_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#enum: Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub items: Option<SchemaDocumentPropertyArray>
}

#[derive(Serialize, Deserialize, GraphQLObject)]
pub struct SchemaDocumentPropertyArray
{
	pub r#type: String,
	pub maximum: i32
}

pub type SchemaRequiredTypes = Vec<String>;