pub struct Mutation;

#[juniper::graphql_object]
impl Mutation
{
	fn alchemy_version() -> &'static str
	{
		return env!("CARGO_PKG_VERSION");
	}
}