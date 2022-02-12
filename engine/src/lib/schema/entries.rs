use crate::lib::database::DATABASE;

use serde::{ Serialize, Deserialize };
use serde_json::{
	value::Value as JsonValue,
	to_value as toJsonValue
};

use rust_arango::AqlQuery;

use crate::lib::database::schema::rule::Rule;

#[derive(Serialize, Deserialize, Default)]
#[derive(Derivative)]
pub struct AlchemyCollectionEntry
{
	pub name: String,
	pub schema: JsonValue,
	#[derivative(Default(value="0"))]
	pub count: u64
}

pub fn create_entry(
	name: String,
	schema_rule: Rule,
) {
	/* Collection entry */
	let alchemy_collection_entry = AlchemyCollectionEntry { 
		name,
		schema: toJsonValue(&schema_rule).unwrap(),
		..Default::default()
	};

	// Create an entry in the alchemy collections
	let alchemy_entry = AqlQuery::builder()
		.query("INSERT @document  INTO @@collection")
		.bind_var("@collection", "alchemy_collections")
		.bind_var("document", toJsonValue(&alchemy_collection_entry).unwrap())
		.build();

	let _alchemy_entry_document: Vec<JsonValue> = DATABASE.database.aql_query(alchemy_entry).unwrap();
}