use super::Property;

use crate::lib::database::Database;
use crate::lib::schema::properties::SchemaDocumentProperty;


use serde_json::{ 
	json, 
	to_value as to_json_value, 
	value::Value as JSONValue,
};
use arangors_lite::AqlQuery;

pub struct Collection
{
	pub name: String,
	pub schema: Vec<SchemaDocumentProperty>,

	arango: Database
}

impl Collection
{
	/// Generate a list of properties and its configurations of the collection
	/// 
	/// For instance a `User` collection the following would be considered properties:
	/// `first_name`, `last_name`, `email`, `password`, the types of these properties
	/// as well as the maximum and minimum length.
	fn get_properties(&self) -> Vec<Property>
	{
		let mut properties: Vec<Property> = vec![];

		let document_properties = &self
			.schema
			.iter()
			.for_each(| property |
			{
				properties.push(Property {
					name: property.name.clone(),
					r#type: property.properties.r#type
				});
			});

		return properties;
	}

	/// Generate a list of relations of the collection
	fn get_relations()
	{
		todo!();
	}

	fn validate_properties(&self, document: JSONValue)
	{
		// Validate document against schema by checking if the properties of the document
		// exist in the schema and if the properties are of the correct type
		document
			.as_object()
			.iter()
			.for_each(| property |
			{
				todo!();
			});
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