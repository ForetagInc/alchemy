use crate::lib::database::Database;

pub struct Context
{
	pub authenticated: bool,
	pub database: Database
}

impl Context
{
	pub async fn new() -> Context
	{
		let database = Database::new().await;

		Context
		{
			authenticated: false,
			database
		}
	}
}

impl juniper::Context for Context {}