use std::env::var as ENV_VAR;

use arangors_lite::Connection as ArangoConnection;
use arangors_lite::collection::options::{
	CreateParameters, 
	CreateOptions as CollectionOptions
};

use anyhow::Error;
use serde_json::{
	value::Value as JsonValue,
	to_value as toJsonValue
};

use arangors_lite::Database as ArangoDatabase;

use crate::lib::schema::collection_types::{
	CollectionSchema,
	CollectionSchemaRule,
	SchemaRequiredTypes
};

use crate::lib::schema::types::document_property::SchemaDocumentProperty;

pub struct Database
{
	pub connection: ArangoConnection,
	pub database: ArangoDatabase
}

impl Database
{
	pub async fn new() -> Database
	{
		let connection = ArangoConnection::establish_basic_auth(
			ENV_VAR("DB_HOST").unwrap_or(String::from("http://localhost:8529")).as_str(), 
			ENV_VAR("DB_USER").unwrap_or(String::from("root")).as_str(),
			ENV_VAR("DB_PASS").unwrap_or(String::from("root01")).as_str(),
		)
		.await
		.unwrap();
		
		let database = connection
			.db("alchemy")
			.await
			.unwrap();

		// let collections = database.accessible_collections().await?;

		// println!("Collections: {:?}", collections);

		Database
		{
			connection,
			database
		}
	}

	pub async fn create_collection(
		&self, 
		name: String,
		properties: Vec<SchemaDocumentProperty>,
		required: Option<SchemaRequiredTypes>
	) -> Result<(), Error> {

		// Create a schema struct to be populated with an empty JSON Map for properties
		let mut schema = CollectionSchema {
			message: String::from(format!("{:?} schema validation failed", name)),
			level: String::from("strict"),
			rule: CollectionSchemaRule {
				r#type: String::from("object"),
				properties: JsonValue::Object(serde_json::Map::new()),
				required,
				additional_properties: false
			}
		};

		// Iterate over the properties and add them to the schema rules
		for property in properties
		{
			schema.rule.properties
				.as_object_mut()
				.unwrap()
				.insert(
					property.name.clone(),
					toJsonValue(property.properties).unwrap(),
				);
		}

		// Create the collection with the schema
		let collection_options = CollectionOptions::builder()
			.name(name.as_str())
			.schema(toJsonValue(schema).unwrap())
			.build();

		self.database.create_collection_with_options(collection_options, CreateParameters::default()).await?;

		Ok(())
	}
}