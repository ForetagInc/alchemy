use juniper::GraphQLObject;

use super::SchemaDocumentProperty;

#[derive(GraphQLObject)]
pub struct Collection
{
	pub name: String,
	pub schema: Vec<SchemaDocumentProperty>
}