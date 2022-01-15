use serde::{ Serialize, Deserialize };

use crate::lib::database::schema::{ 
	SchemaProperty,
	SchemaPropertyType, 
	SchemaNativeTypeArray
 };

#[derive(Serialize, Deserialize, PartialEq)]
pub struct SchemaDocumentProperty
{
	pub name: String,
	pub values: SchemaDocumentPropertyValues
}

#[derive(Serialize, Deserialize, PartialEq, Default)]
pub struct SchemaDocumentPropertyValues
{
	pub r#type: SchemaPropertyType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub min_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#enum: Option<Vec<String>>
}

impl SchemaDocumentPropertyValues
{
	pub fn new() -> Self
	{
		Default::default()
	}
}

impl From<SchemaDocumentPropertyValues> for SchemaProperty
{
	fn from(values: SchemaDocumentPropertyValues) -> Self
	{
		let mut property = SchemaProperty::new();
		property.r#type = Some(values.r#type.to_string());

		match values.r#type
		{
			SchemaPropertyType::Array => {
				property.items = Some(SchemaNativeTypeArray {
					r#type: values.r#type.into(),
					maximum: values.max_length.unwrap_or(0)
				});
			},
			SchemaPropertyType::Enum => {
				property.r#enum = values.r#enum;
			},
			// Default for string, integer and boolean
			_ => {
				property.r#type = Some(values.r#type.to_string());
				property.min_length = values.min_length;
				property.max_length =  values.max_length;
			}
		}

		return property;
	}
}