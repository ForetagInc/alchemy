use serde::{ Serialize, Deserialize };
use super::Rule;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseSchema
{
	pub message: String,
	pub level: String,
	pub rule: Rule
}