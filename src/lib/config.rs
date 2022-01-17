use std::{fs::File, io::Read};

use anyhow::Result;
use serde::Deserialize;
use lazy_static::lazy_static;

#[derive(Deserialize, Debug)]
pub struct Config
{
	pub db_host: String,
	pub db_user: String,
	pub db_pass: String,
	pub db_name: String,
	
	pub rust_env: String
}

impl Config
{
	pub fn is_production(&self) -> bool
	{
		self.rust_env == "production"
	}

	pub fn is_development(&self) -> bool
	{
		!self.is_production()
	}
}

fn load_config() -> Result<Config>
{
	let env = envy::from_env::<Config>();

	match env
	{
		Ok(config) => Ok(config),
		Err(_) =>
		{
			// Load from .env file

			let mut file = File::open(".env")?;
			let mut content = String::new();
			
			file.read_to_string(&mut content)?;

			for line in content.lines()
			{
				let pair = line.split('=').collect::<Vec<&str>>();

				let (key, value) = match &pair[..] {
					&[key, value] => (key, value),
					_ => panic!("Expected env variable pairs, got {}", content)
				};

				std::env::set_var(key, value);
			}

			match envy::from_env::<Config>() {
				Ok(config) => Ok(config),
				Err(e) => panic!("Failed to read the config from env: {}", e),
			}
		}
	}
}

lazy_static! {
	pub static ref CONFIG: Config = load_config().unwrap();
}