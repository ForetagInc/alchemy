use crate::lib::database::Database;
use crate::lib::schema::properties::SchemaDocumentProperty;

use serde_json::{
	from_value,
	value::Value as JSONValue
};
use jsonschema::{ is_valid, JSONSchema };

use rust_arango::{ AqlQuery, collection::response::Properties };

pub struct Collection
{
	pub name: String,
	pub properties: Vec<SchemaDocumentProperty>,

	arango: Database
}

impl Collection
{
	/// Get the properties of an Arango collection
	pub async fn get_arango_properties(&self) -> Properties
	{
		return self.arango
			.database
			.collection(self.name.as_str())
			.await
			.unwrap()
			.properties()
			.await
			.unwrap();
	}

	/// Get the schema of an Arango collection from the properties
	pub async fn get_json_schema(&self) -> JSONValue
	{
		return from_value(
			self
				.get_arango_properties()
				.await
				.info
				.schema
				.unwrap()
		)
		.unwrap();
	}

	/// Generate a list of relations of the collection
	fn get_relations()
	{
		todo!();
	}

	/// Validate document against schema by checking if the properties of the document
	/// exist in the schema and if the properties are of the correct type
	pub async fn validate_properties(&self, document: JSONValue) -> bool
	{
		let schema = self.get_json_schema().await;

		let compiled = JSONSchema::compile(&schema)
			.expect("Schema could not be compiled");

		return is_valid(&schema, &document);
	}

	pub async fn create_document(&self, document: JSONValue) -> Vec<serde_json::Value>
	{
		// TODO: need to validate the document against the schema of the collection

		let aql = AqlQuery::builder()
			.query("INSERT @document INTO @@collection
				LET result = NEW RETURN result")
			.bind_var("@collection", self.name.clone())
			.bind_var("document", document)
			.build();

		let result: &Vec<JSONValue> = &self
			.arango
			.database
			.aql_query(aql)
			.await
			.unwrap();

		return result.to_vec();
	}

	pub async fn read_document()
	{
		todo!()
	}

	pub async fn update_document()
	{
		todo!()
	}

	pub async fn delete_document()
	{
		todo!()
	}
}