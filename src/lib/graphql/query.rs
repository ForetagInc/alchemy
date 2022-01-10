use serde_json::json;

pub struct Query;

#[juniper::graphql_object]
impl Query
{
	fn alchemy_version() -> &'static str
	{
		return env!("CARGO_PKG_VERSION");
	}
}