use serde::{ Serialize, Deserialize };

use super::{ SchemaPropertyType, SchemaNativeType };

/// The schema property
#[derive(Serialize, Deserialize)]
pub struct SchemaProperty
{
	pub r#type: SchemaPropertyType,
	pub min_length: Option<i32>,
	pub max_length: Option<i32>,
	pub items: Option<Vec<SchemaNativeType>>
}