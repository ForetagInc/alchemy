use serde::{ Serialize, Deserialize };

/// The schema for the API
#[derive(Serialize, Deserialize, Debug)]
pub struct APISchema
{
	pub collections: Vec<String>
}