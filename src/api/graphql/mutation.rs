use super::Context;

use crate::lib::database::schema::{
	SchemaPropertyType,
	SchemaNativeType
};

use crate::lib::schema::{
	SchemaDocumentProperty, 
	SchemaDocumentPropertyValues,
};

pub struct Mutation;

#[juniper::graphql_object(context = Context)]
impl Mutation
{
	pub async fn create_collection(
		context: &Context,
		#[graphql] name: String,
		// #[graphql] properties: Vec<SchemaDocumentProperty>,
	) -> bool {
		let properties = vec![
			SchemaDocumentProperty {
				name: "firstName".to_string(),
				values: SchemaDocumentPropertyValues {
					r#type: SchemaPropertyType::String,
					max_length: Some(30),
					..Default::default()
				}
			},
			SchemaDocumentProperty {
				name: "lastName".to_string(),
				values: SchemaDocumentPropertyValues {
					r#type: SchemaPropertyType::String,
					min_length: Some(6),
					max_length: Some(30),
					..Default::default()
				}
			},
			SchemaDocumentProperty {
				name: "tags".to_string(),
				values: SchemaDocumentPropertyValues {
					r#type: SchemaPropertyType::Array,
					array_type: Some(SchemaNativeType::String),
					max_length: Some(5),
					..Default::default()
				}
			},
		];

		if let Ok(_) = context
			.database
			.create_collection(name, properties, None)
			.await {
				return true;
			} else {
				return false;
			}
	}
}