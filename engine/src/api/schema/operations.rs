use convert_case::Casing;
use serde_json::Value as JsonValue;

use crate::lib::database::api::GraphQLType;

trait Operation {
	fn call(graphql_type: &GraphQLType) -> JsonValue;

	fn get_collection_name(type_name: String) -> String {
		type_name.to_case(convert_case::Case::Snake).pluralize()
	}
}

pub struct Get;

impl Operation for Get {
	fn call(graphql_type: &GraphQLType) -> JsonValue {
		let collection = p
	}
}