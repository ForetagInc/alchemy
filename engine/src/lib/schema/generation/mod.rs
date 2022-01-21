use crate::lib::database::Database;

pub async fn generate_collections() 
{
	let arango = Database::new().await;

	let mut schema_collections: Vec<String> = Vec::new();
	
	let collections = arango.database.accessible_collections().await.unwrap();

	// Iterate through the collections and check if they are a schema collection and not
	// a system collection from ArangoDB
	// TODO: consider edge / graph collections
	for collection in collections.iter()
	{
		if !collection.is_system
		{
			schema_collections.push(collection.name.clone());
		}
	}

	println!("Collections: {:?}", schema_collections);
}