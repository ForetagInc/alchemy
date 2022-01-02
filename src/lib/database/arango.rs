use std::env::var as ENV_VAR;

use arangors_lite::Connection as ArangoConnection;
use arangors_lite::collection::options::{
	CreateParameters, 
	CreateOptions as CollectionOptions
};

use anyhow::Error;
use serde_json::json;

use arangors_lite::Database as ArangoDatabase;

use crate::lib::schema::types::{ 
	SchemaDocumentProperty,
	SchemaRequiredTypes
};

pub struct Database
{
	connection: ArangoConnection,
	database: ArangoDatabase
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
		properties: SchemaDocumentProperty,
		required: Option<SchemaRequiredTypes>
	) -> Result<(), Error> {
		let schema = json!({
			"message": "Test validation failed",
			"level": "strict",
			"rule": {
				"type": "object",
				"properties": properties
			},
			"required": required.unwrap_or(vec![]),
			"additionalProperties": false
		});

		let collection_options = CollectionOptions::builder()
			.name(name.as_str())
			.schema(schema)
			.build();

		let collection = self.database.create_collection_with_options(collection_options, CreateParameters::default()).await?;

		println!("Created collection {:?}", collection);

		Ok(())
	}
}