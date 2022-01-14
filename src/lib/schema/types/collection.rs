use super::document_property::SchemaDocumentProperty;

#[derive(GraphQLObject)]
pub struct Collection
{
	pub name: String,
	pub schema: Vec<SchemaDocumentProperty>
}