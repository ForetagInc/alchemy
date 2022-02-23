use super::Context;

use crate::lib::schema::SchemaDocumentProperty;
use crate::lib::database::arango::{ create_collection, delete_collection };

pub struct Mutation;

#[juniper::graphql_object(context = Context)]
impl Mutation
{
	pub async fn create_collection(
		_context: &Context,
		#[graphql] name: String,
		#[graphql] properties: Vec<SchemaDocumentProperty>,
	) -> bool {
		return if let Ok(_) = create_collection(name, properties).await {
			true
		} else {
			false
		}
	}

	pub async fn delete_collection(
		_context: &Context,
		#[graphql] name: String
	) -> bool {
		return if let Ok(_) = delete_collection(name).await {
			true
		} else {
			false
		}
	}
}