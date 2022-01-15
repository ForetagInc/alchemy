use serde::{ Serialize, Deserialize };

/// The native types for Arango to store
#[derive(Serialize, Deserialize, PartialEq, Default)]
pub enum SchemaNativeType
{
	#[default] String,
	Integer,
	Boolean
}