use core::fmt;

use serde::{ Serialize, Deserialize };

use super::SchemaNativeType;

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

impl fmt::Display for SchemaPropertyType
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self
		{
			SchemaPropertyType::String => write!(f, "string"),
			SchemaPropertyType::Integer => write!(f, "integer"),
			SchemaPropertyType::Array => write!(f, "array"),
			SchemaPropertyType::Boolean => write!(f, "boolean"),
			SchemaPropertyType::Enum => write!(f, "enum"),
		}
	}
}

impl From<SchemaPropertyType> for SchemaNativeType
{
	fn from(property_type: SchemaPropertyType) -> Self
	{
		match property_type
		{
			SchemaPropertyType::String => SchemaNativeType::String,
			SchemaPropertyType::Integer => SchemaNativeType::Integer,
			SchemaPropertyType::Array => SchemaNativeType::String,
			SchemaPropertyType::Boolean => SchemaNativeType::Boolean,
			SchemaPropertyType::Enum => SchemaNativeType::String,
		}
	}
}