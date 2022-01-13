pub mod collection;

use juniper::{ FromInputValue, InputValue, marker::IsInputType };
use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionSchema
{
	pub message: String,
	pub level: String,
	pub rule: CollectionSchemaRule
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSchemaRule
{
	pub r#type: String,
	pub properties: serde_json::Value,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub required: Option<SchemaRequiredTypes>,
	pub additional_properties: bool
}

#[derive(Default, Serialize, Deserialize, Debug, GraphQLObject)]
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

#[derive(Default, Serialize, Deserialize, Debug, GraphQLObject)]
pub struct SchemaDocumentProperty
{
	pub name: String,
	pub properties: SchemaDocumentPropertyValues
}

impl FromInputValue for SchemaDocumentProperty
{
	fn from_input_value(v: &InputValue) -> Option<Self>
	{
		let object = v.to_object_value().unwrap();

		let name = object
			.get("name")
			.and_then(|t| t.as_string_value())
			.unwrap_or("")
			.to_string();
		
		let properties = object
			.get("properties")
			.and_then(|t| t.to_object_value())
			.unwrap()
			.clone();

		Some(SchemaDocumentProperty {
			name,
			properties: SchemaDocumentPropertyValues 
			{
				r#type: properties
					.get("type")
					.and_then(|t| t.as_string_value())
					.map(|v| v.to_string()),
				
				min_length: properties
					.get("minLength")
					.and_then(|t| t.as_int_value())
					.map(|t| t as i32),

				max_length: properties
						.get("maxLength")
						.and_then(|t| t.as_int_value())
						.map(|v| v as i32),
				
				r#enum: properties
					.get("enum")
					.and_then(|t| t.to_list_value())
					.map(| t| 
						t
							.iter()
							.map(| x| 
								x
									.as_string_value()
									.unwrap_or("").to_string()
							)
						.collect()
					),
				
				items: properties
					.get("items")
					.and_then(|t| t.to_object_value())
					.map(|t| SchemaDocumentPropertyArray {
						r#type: t
								.get("type")
								.and_then(|x| x.as_string_value())
								.map(|x| x.to_string())
								.unwrap_or("".to_string()),
							
						maximum: t
								.get("maximum")
								.and_then(|x| x.as_int_value())
								.map(|x| x as i32)
								.unwrap_or(0)
					})
			}
		})
	}
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct SchemaDocumentPropertyArray
{
	pub r#type: String,
	pub maximum: i32
}

pub type SchemaRequiredTypes = Vec<String>;