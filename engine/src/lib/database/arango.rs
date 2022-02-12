use rust_arango::collection::options::{
	CreateParameters, 
	CreateOptions as CollectionOptions
};

use anyhow::Error;

use serde_json::{
	value::Value as JsonValue,
	to_value as toJsonValue
};

use crate::lib::database::DATABASE;
use crate::lib::schema::SchemaDocumentProperty;
use crate::lib::database::schema::{ DatabaseSchema, Rule, SchemaProperty };

pub async fn create_collection(
	name: String,
	properties: Vec<SchemaDocumentProperty>,
	required: Option<Vec<String>>
) -> Result<(), Error> {
	let db = DATABASE.get().await.database.clone();

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

	db.create_collection_with_options(collection_options, CreateParameters::default()).await?;

	// /* Collection entry */
	// let alchemy_collection_entry = AlchemyCollectionEntry { 
	// 	name,
	// 	schema: toJsonValue(&schema.rule).unwrap(),
	// 	..Default::default()
	// };

	// // Create an entry in the alchemy collections
	// let alchemy_entry = AqlQuery::builder()
	// 	.query("INSERT @document  INTO @@collection")
	// 	.bind_var("@collection", "alchemy_collections")
	// 	.bind_var("document", toJsonValue(&alchemy_collection_entry).unwrap())
	// 	.build();

	// let _alchemy_entry_document: Vec<JsonValue> = self.database.aql_query(alchemy_entry).await.unwrap();

	Ok(())
}

pub async fn delete_collection(
	name: String
) -> Result<(), Error> {

	let db = DATABASE.get().await.database.clone();

	db.drop_collection(name.as_str()).await?;

	Ok(())
}