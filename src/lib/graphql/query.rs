pub struct Query;

#[juniper::graphql_object]
impl Query
{
	fn api_version() -> &'static str
	{
		return env!("CARGO_PKG_VERSION");
	}
}