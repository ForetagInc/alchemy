use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::lib::CONFIG;

use rust_arango::{
	Connection as ArangoConnection,
	Database as ArangoDatabase
};

pub struct ArangoDB
{
	pub connection: ArangoConnection,
	pub database: ArangoDatabase,
}

impl ArangoDB
{
	pub fn new() -> ArangoDB
	{
		let connection = ArangoConnection::establish_basic_auth(
			&CONFIG.db_host.as_str(),
			&CONFIG.db_user.as_str(),
			&CONFIG.db_pass.as_str(),
		)
		.unwrap();
		
		let database = connection
			.db(&CONFIG.db_name.as_str())
			.unwrap();

		ArangoDB
		{
			connection,
			database
		}
	}

	pub fn initialize(&self)
	{
		// Get all existing collections
		let collections = self.database.accessible_collections().unwrap();

		// Iterate through the collections and check if there is a alchemy collections setup
		for collection in collections
		{
			if collection.name == String::from("alchemy_collections")
			{
				return;
			}
		}

		// Create the collection
		self.database.create_collection("alchemy_collections");
	}
}


lazy_static::lazy_static! {
    pub static ref DATABASE: Arc<ArangoDB> = Arc::new(ArangoDB::new());
}