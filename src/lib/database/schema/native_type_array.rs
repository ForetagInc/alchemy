use serde::{ Serialize, Deserialize };

use super::SchemaNativeType;

/// Schema native array for Arango
#[derive(Serialize, Deserialize)]
pub struct SchemaNativeTypeArray
{
	pub r#type: SchemaNativeType,
	pub maximum: i32
}