#[derive(Default, Clone)]
pub struct ContextObject
{
	authenticated: bool
}

impl ContextObject
{
	pub fn new() -> ContextObject
	{
		ContextObject 
		{
			authenticated: false
		}
	}

	pub fn ctx() -> ()
	{
		()
	}
}

impl juniper::Context for ContextObject {}