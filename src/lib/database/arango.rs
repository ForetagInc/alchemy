use std::env::var as ENV_VAR;

use arangors_lite::Connection;
use arangors_lite::collection::options::{
	CreateParameters, 
	CreateOptions as CollectionOptions
};

use anyhow::Error;
use serde_json::json;

pub struct Database;

impl Database
{
	async fn new() -> Result<(), Error>
	{
		let connection = Connection::establish_basic_auth(
			ENV_VAR("DB_HOST").unwrap_or(String::from("http://localhost:8529")).as_str(), 
			ENV_VAR("DB_USER").unwrap_or(String::from("root")).as_str(),
			ENV_VAR("DB_PASS").unwrap_or(String::from("root01")).as_str(),
		).await?;
		
		let database = connection.db("alchemy").await?;

		let schema = json!({
			"message": "Test validation failed",
			"level": "strict",
			"rule": {
				"type": "object",
				"properties": {
					"firstName": {
						"type": "string",
						"minLength": 2,
						"maxLength": 30
					},
					"lastName": {
						"type": "string",
						"minLength": 2,
						"maxLength": 30
					},
					"role": {
						"enum": [
							"lead",
							"customer",
						]
					}
				}
			},
			"required": [
				"firstName",
				"lastName"
			],
			"additionalProperties": false
		});

		let collection_options = CollectionOptions::builder()
			.name("test")
			.schema(schema)
			.build();

		let test = database.create_collection_with_options(collection_options, CreateParameters::default()).await?;

		// println!("Created collection {:?}", test);

		// let collections = database.accessible_collections().await?;

		// println!("Collections: {:?}", collections);

		Ok(())
	}
}