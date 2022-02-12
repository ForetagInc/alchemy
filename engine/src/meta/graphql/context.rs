use crate::lib::database::DATABASE;
use rust_arango::Database as ArangoDatabase;

pub struct Context
{
	pub authenticated: bool,
	pub database: ArangoDatabase
}

impl Context
{
	pub async fn new() -> Context
	{
		Context
		{
			authenticated: false,
			database: DATABASE.get().await.database.clone()
		}
	}
}

impl juniper::Context for Context {}