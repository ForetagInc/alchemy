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

use crate::lib::CONFIG;
use crate::lib::schema::SchemaDocumentProperty;
use crate::lib::database::schema::{ Schema, Rule, SchemaProperty };

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
			&CONFIG.db_host.as_str(), 
			&CONFIG.db_user.as_str(),
			&CONFIG.db_pass.as_str(),
		)
		.await
		.unwrap();
		
		let database = connection
			.db(&CONFIG.db_name.as_str())
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
		required: Option<Vec<String>>
	) -> Result<(), Error> {

		// Create a schema struct to be populated with an empty JSON Map for properties
		let mut schema = Schema {
			message: String::from("Schema validation failed"),
			level: String::from("strict"),
			rule: Rule {
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
					toJsonValue(SchemaProperty::from(property.values))
						.unwrap(),
				);
		}

		println!("{:?}", toJsonValue(schema.clone()).unwrap().to_string());

		// Create the collection with the schema
		let collection_options = CollectionOptions::builder()
			.name(name.as_str())
			.schema(toJsonValue(schema).unwrap())
			.build();

		self.database.create_collection_with_options(collection_options, CreateParameters::default()).await?;

		Ok(())
	}

	pub async fn delete_collection(
		&self,
		name: String
	) -> Result<(), Error> {

		self.database.drop_collection(name.as_str()).await?;

		Ok(())
	}
}