use serde::{ Serialize, Deserialize };

/// The schema property type of the collection
#[derive(Serialize, Deserialize, PartialEq, Default)]
pub enum SchemaPropertyType
{
	#[default] String,
	Integer,
	Array,
	Boolean,
	Enum
}