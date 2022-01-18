use serde::{ Serialize, Deserialize };

/// Schema native array for Arango
#[derive(Serialize, Deserialize, PartialEq)]
pub struct SchemaNativeTypeArray
{
	pub r#type: String,
	pub maximum: Option<i32>
}