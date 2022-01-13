use crate::lib::graphql::Context;

use crate::lib::schema::types::{
	SchemaDocumentProperty,
	SchemaDocumentPropertyValues,
	SchemaDocumentPropertyArray
};

pub struct Mutation;

#[juniper::graphql_object(context = Context)]
impl Mutation
{
	pub async fn create_collection(
		context: &Context,
		#[graphql] name: String,
		#[graphql] properties: Vec<SchemaDocumentProperty>,
	) -> bool {
		let schema = vec![
			SchemaDocumentProperty {
				name: "firstName".to_string(),
				properties: SchemaDocumentPropertyValues {
					r#type: Some(String::from("string")),
					min_length: Some(4),
					max_length: Some(30),
					r#enum: None,
					items: None
				}
			},
			SchemaDocumentProperty {
				name: "lastName".to_string(),
				properties: SchemaDocumentPropertyValues {
					r#type: Some(String::from("string")),
					min_length: Some(6),
					max_length: Some(60),
					r#enum: None,
					items: None
				}
			},
			SchemaDocumentProperty {
				name: "tags".to_string(),
				properties: SchemaDocumentPropertyValues {
					r#type: Some(String::from("array")),
					min_length: None,
					max_length: None,
					r#enum: None,
					items: Some(SchemaDocumentPropertyArray {
						r#type: "number".to_string(),
						maximum: 10
					})
				}
			}
		];

		if let Ok(_) = context
			.database
			.create_collection(name, schema, None)
			.await {
				return true;
			} else {
				return false;
			}
	}
}