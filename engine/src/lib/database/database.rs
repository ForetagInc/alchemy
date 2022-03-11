use async_once::AsyncOnce;
use std::sync::Arc;

use crate::lib::CONFIG;

use rust_arango::{Connection as ArangoConnection, Database as ArangoDatabase};

pub struct ArangoDB {
    pub connection: ArangoConnection,
    pub database: ArangoDatabase,
}

impl ArangoDB {
    pub async fn new() -> ArangoDB {
        let connection = ArangoConnection::establish_basic_auth(
            &CONFIG.db_host.as_str(),
            &CONFIG.db_user.as_str(),
            &CONFIG.db_pass.as_str(),
        )
        .await
        .unwrap();

        let database = connection.db(&CONFIG.db_name.as_str()).await.unwrap();

        ArangoDB {
            connection,
            database,
        }
    }

    // pub async fn initialize(&self)
    // {
    // 	// Get all existing collections
    // 	let collections = self.database.accessible_collections().await.unwrap();

    // 	// Iterate through the collections and check if there is a alchemy collections setup
    // 	for collection in collections
    // 	{
    // 		if collection.name == String::from("alchemy_collections")
    // 		{
    // 			return;
    // 		}
    // 	}

    // 	// Create the collection
    // 	self.database.create_collection("alchemy_collections").await.unwrap();
    // }
}

lazy_static::lazy_static! {
    pub static ref DATABASE: AsyncOnce<Arc<ArangoDB>> = AsyncOnce::new(async {
           Arc::new(ArangoDB::new().await)
       });
}
