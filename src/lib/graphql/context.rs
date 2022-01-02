#[derive(Default, Clone)]
pub struct Context
{
	authenticated: bool
}

impl Context
{
	pub fn new() -> Context
	{
		Context
		{
			authenticated: false
		}
	}

	pub fn ctx() -> ()
	{
		()
	}
}

impl juniper::Context for Context {}