use serde::{ Serialize, Deserialize };

/// The native types for Arango to store
#[derive(Serialize, Deserialize)]
pub enum SchemaNativeType
{
	String,
	Integer,
	Array,
	Boolean
}