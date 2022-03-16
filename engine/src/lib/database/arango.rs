use rust_arango::collection::options::{CreateOptions as CollectionOptions, CreateParameters};

use anyhow::Error;

use serde_json::{to_value as toJsonValue, value::Value as JsonValue};

use crate::lib::database::schema::{DatabaseSchema, Rule, SchemaProperty};
use crate::lib::database::DATABASE;
use crate::lib::schema::{create_entry, delete_entry, SchemaDocumentProperty};

pub async fn create_collection(
	name: String,
	properties: Vec<SchemaDocumentProperty>,
) -> Result<(), Error> {
	let db = DATABASE.get().await.database.clone();

	// Create a schema struct to be populated with an empty JSON Map for properties
	let mut schema = DatabaseSchema {
		message: String::from("Schema validation failed"),
		level: String::from("strict"),
		rule: Rule {
			r#type: String::from("object"),
			properties: JsonValue::Object(serde_json::Map::new()),
			required: Vec::new(),
			additional_properties: false,
		},
	};

	// Iterate over the properties and add them to the schema rules
	for property in properties {
		schema.rule.properties.as_object_mut().unwrap().insert(
			property.name.clone(),
			toJsonValue(SchemaProperty::from(property.values)).unwrap(),
		);

		if property.required {
			schema.rule.required.push(property.name);
		}
	}

	// println!("{:?}", toJsonValue(schema.clone()).unwrap().to_string());

	// Create the collection with the schema
	let collection_options = CollectionOptions::builder()
		.name(name.as_str())
		.schema(toJsonValue(&schema).unwrap())
		.build();

	db.create_collection_with_options(collection_options, CreateParameters::default())
		.await?;

	create_entry(name, schema.rule).await;

	Ok(())
}

pub async fn delete_collection(name: String) -> Result<(), Error> {
	let db = DATABASE.get().await.database.clone();

	db.drop_collection(name.as_str()).await?;

	delete_entry(name).await;

	Ok(())
}
