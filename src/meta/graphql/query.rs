use super::Context;

pub struct Query;

#[juniper::graphql_object(context = Context)]
impl Query
{
	fn alchemy_version() -> &'static str
	{
		return env!("CARGO_PKG_VERSION");
	}
}