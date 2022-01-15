use serde::{ Serialize, Deserialize };

use crate::lib::database::schema::{ 
	SchemaProperty,
	SchemaNativeType,
	SchemaPropertyType, 
	SchemaNativeTypeArray
 };

#[derive(Serialize, Deserialize, PartialEq, GraphQLInputObject)]
pub struct SchemaDocumentProperty
{
	pub name: String,
	pub values: SchemaDocumentPropertyValues
}

#[derive(Serialize, Deserialize, PartialEq, Default, GraphQLInputObject)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDocumentPropertyValues
{
	pub r#type: SchemaPropertyType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub min_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_length: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#enum: Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub array_type: Option<SchemaNativeType>
}

impl From<SchemaDocumentPropertyValues> for SchemaProperty
{
	fn from(values: SchemaDocumentPropertyValues) -> Self
	{
		let mut property = SchemaProperty::new();
		property.r#type = Some(values.r#type.as_str());

		match values.r#type
		{
			SchemaPropertyType::Array => {
				property.items = Some(SchemaNativeTypeArray {
					r#type: values
							.array_type
							.unwrap_or(SchemaNativeType::String)
							.as_str(),
					maximum: values.max_length.unwrap_or(0)
				});
			},
			SchemaPropertyType::Enum => {
				property.r#enum = values.r#enum;
			},
			// Default for string, integer and boolean
			_ => {
				property.min_length = values.min_length;
				property.max_length =  values.max_length;
			}
		}

		return property;
	}
}