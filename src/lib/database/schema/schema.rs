use serde::{ Serialize, Deserialize };
use super::Rule;

#[derive(Serialize, Deserialize)]
pub struct Schema
{
	pub message: String,
	pub level: String,
	pub rule: Rule
}