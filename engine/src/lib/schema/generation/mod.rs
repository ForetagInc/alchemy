use crate::lib::{
	database::Database,
	database::schema::{ DatabaseSchema, SchemaProperty },
	schema::{ SchemaDocumentProperty, SchemaDocumentPropertyValues }
};

use rust_arango::AqlQuery;

pub async fn generate_collections() 
{
	let arango = Database::new().await;

	let schema_collections: Vec<SchemaDocumentProperty> = Vec::new();
	
	let collections = arango.database.accessible_collections().await.unwrap();

	// Iterate through the collections and check if they are a schema collection and not
	// a system collection from ArangoDB
	// TODO: consider edge / graph collections
	for collection in collections.iter()
	{
		if !collection.is_system
		{
			let name = collection.name.clone();

			// Fetch the schema from the database
			let properties = arango
				.database
				.collection(name.as_str())
				.await
				.unwrap()
				.properties()
				.await
				.unwrap();

			let schema: DatabaseSchema = serde_json::from_value(properties.info.schema.unwrap()).unwrap();

			println!("Schema: {:?}", schema);
		}
	}

	// println!("Collections: {:?}", schema_collections);
}