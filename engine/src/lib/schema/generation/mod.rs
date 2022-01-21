use crate::lib::{
	database::Database, 
	schema::{ SchemaDocumentProperty, SchemaDocumentPropertyValues }
};

pub async fn generate_collections() 
{
	let arango = Database::new().await;

	let mut schema_collections: Vec<SchemaDocumentProperty> = Vec::new();
	
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
			let instance = arango.database.collection(name.as_str()).await.unwrap();

			println!("Collection {} with data {:?}", name, instance.properties().await.unwrap());

			// schema_collections.push(SchemaDocumentProperty {
			// 	name: collection.name.clone(),
			// 	values: SchemaDocumentPropertyValues {
					
			// 	}
			// });
		}
	}

	// println!("Collections: {:?}", schema_collections);
}