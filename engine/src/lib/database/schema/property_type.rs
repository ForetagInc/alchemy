use serde::{Deserialize, Serialize};

use super::SchemaNativeType;

/// The schema property type of the collection
#[derive(Serialize, Deserialize, PartialEq, Default, GraphQLEnum)]
pub enum SchemaPropertyType {
	#[default]
	String,
	Integer,
	Array,
	Boolean,
	Enum,
}

impl SchemaPropertyType {
	pub fn as_str(&self) -> String {
		match self {
			SchemaPropertyType::String => String::from("string"),
			SchemaPropertyType::Integer => String::from("integer"),
			SchemaPropertyType::Array => String::from("array"),
			SchemaPropertyType::Boolean => String::from("boolean"),
			SchemaPropertyType::Enum => String::from("enum"),
		}
	}
}

impl From<SchemaPropertyType> for SchemaNativeType {
	fn from(property_type: SchemaPropertyType) -> Self {
		match property_type {
			SchemaPropertyType::String => SchemaNativeType::String,
			SchemaPropertyType::Integer => SchemaNativeType::Integer,
			SchemaPropertyType::Array => SchemaNativeType::String,
			SchemaPropertyType::Boolean => SchemaNativeType::Boolean,
			SchemaPropertyType::Enum => SchemaNativeType::String,
		}
	}
}
