use serde::{ Serialize, Deserialize };

/// The schema property type of the collection
#[derive(Serialize, Deserialize)]
pub enum SchemaPropertyType
{
	String,
	Integer,
	Array,
	Boolean,
	Enum
}