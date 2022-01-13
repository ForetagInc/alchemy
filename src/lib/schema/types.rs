pub mod collection;

use serde::{ Serialize, Deserialize };

pub struct CollectionSchema
{
	message: String,
	level: String,
	rule: CollectionSchemaRule
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSchemaRule
{
	r#type: String,
	properties: Vec<SchemaDocumentProperty>,
	#[serde(skip_serializing_if = "Option::is_none")]
	required: Option<Vec<String>>,
	additional_properties: bool
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

#[derive(Default, Serialize, Deserialize, GraphQLObject)]
pub struct SchemaDocumentProperty
{
	pub name: String,
	pub properties: SchemaDocumentPropertyValues
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct SchemaDocumentPropertyArray
{
	pub r#type: String,
	pub maximum: i32
}

pub type SchemaRequiredTypes = Vec<String>;