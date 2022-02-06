use rust_arango::{
	Connection as ArangoConnection,
	AqlQuery
};
use rust_arango::collection::options::{
	CreateParameters, 
	CreateOptions as CollectionOptions
};

use anyhow::Error;

use serde::{ Serialize, Deserialize };
use serde_json::{
	value::Value as JsonValue,
	to_value as toJsonValue
};

use rust_arango::Database as ArangoDatabase;

use crate::lib::CONFIG;
use crate::lib::schema::SchemaDocumentProperty;
use crate::lib::database::schema::{ DatabaseSchema, Rule, SchemaProperty };

pub struct Database
{
	pub connection: ArangoConnection,
	pub database: ArangoDatabase
}

#[derive(Serialize, Deserialize)]
pub struct AlchemyCollectionEntry
{
	pub name: String,
	pub schema: JsonValue
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

		Database
		{
			connection,
			database
		}
	}

	pub async fn initialize(&self)
	{
		// Get all existing collections
		let collections = self.database.accessible_collections().await.unwrap();

		// Iterate through the collections and check if there is a alchemy collections setup
		for collection in collections
		{
			if collection.name == String::from("alchemy_collections")
			{
				return;
			}
		}

		// Create the collection
		self.database.create_collection("alchemy_collections").await;
	}

	pub async fn create_collection(
		&self, 
		name: String,
		properties: Vec<SchemaDocumentProperty>,
		required: Option<Vec<String>>
	) -> Result<(), Error> {

		// Create a schema struct to be populated with an empty JSON Map for properties
		let mut schema = DatabaseSchema {
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

		// println!("{:?}", toJsonValue(schema.clone()).unwrap().to_string());

		// Create the collection with the schema
		let collection_options = CollectionOptions::builder()
			.name(name.as_str())
			.schema(toJsonValue(&schema).unwrap())
			.build();

		self.database.create_collection_with_options(collection_options, CreateParameters::default()).await?;

		/* Collection entry */
		let alchemy_collection_entry = AlchemyCollectionEntry { 
			name,
			schema: toJsonValue(&schema.rule).unwrap()
		};

		// Create an entry in the alchemy collections
		let alchemy_entry = AqlQuery::builder()
			.query("INSERT @document  INTO @@collection")
			.bind_var("@collection", "alchemy_collections")
			.bind_var("document", toJsonValue(&alchemy_collection_entry).unwrap())
			.build();

		let _alchemy_entry_document: Vec<JsonValue> = self.database.aql_query(alchemy_entry).await.unwrap();

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