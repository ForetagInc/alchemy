use convert_case::Casing;
use serde_json::Value as JsonValue;

use crate::lib::database::api::GraphQLType;

pub trait Operation {
    fn call(graphql_type: &GraphQLType) -> JsonValue;

    fn get_collection_name(type_name: String) -> String {
        pluralizer::pluralize(
            type_name.to_case(convert_case::Case::Snake).as_str(),
            2,
            false,
        )
    }

    fn get_operation_name(type_name: String) -> String;
}

pub struct Get;

impl Operation for Get {
    fn call(graphql_type: &GraphQLType) -> JsonValue {
        let collection = Self::get_collection_name((&graphql_type.name).clone());

        println!("{}", collection);

        todo!()
    }

    fn get_operation_name(type_name: String) -> String {
        format!(
            "get{}",
            pluralizer::pluralize(
                type_name.to_case(convert_case::Case::Pascal).as_str(),
                2,
                false
            )
        )
    }
}
