use super::Context;

use crate::lib::schema::SchemaDocumentProperty;

pub struct Mutation;

#[juniper::graphql_object(context = Context)]
impl Mutation
{
	pub async fn create_collection(
		context: &Context,
		#[graphql] name: String,
		#[graphql] properties: Vec<SchemaDocumentProperty>,
	) -> bool {
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