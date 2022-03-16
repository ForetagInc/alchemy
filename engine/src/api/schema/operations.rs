use std::collections::HashMap;
use std::sync::Mutex;
use convert_case::Casing;
use lazy_static::lazy_static;
use serde_json::Value as JsonValue;

use crate::lib::database::api::GraphQLType;

type OperationClosure = fn(&GraphQLType) -> JsonValue;

pub struct OperationRegistry
{
	operations: HashMap<String, OperationClosure>,
}

impl OperationRegistry
{
	pub fn new() -> OperationRegistry {
		OperationRegistry {
			operations: HashMap::new()
		}
	}

	pub fn register_entity(&mut self, type_name: &str) -> Vec<Option<String>>
	{
		vec![
			self.register::<Get>(type_name)
		]
	}

	fn register<T>(&mut self, type_name: &str) -> Option<String>
		where
			T: Operation
	{
		let key = T::get_operation_name(type_name);

		self.operations.insert(key.clone(), T::call as OperationClosure).map(|_| key)
	}
}

pub trait Operation {
	fn call(graphql_type: &GraphQLType) -> JsonValue;

	fn get_collection_name(type_name: &str) -> String {
		pluralizer::pluralize(
			type_name.to_case(convert_case::Case::Snake).as_str(),
			2,
			false,
		)
	}

	fn get_operation_name(type_name: &str) -> String;
}

pub struct Get;

impl Operation for Get {
	fn call(graphql_type: &GraphQLType) -> JsonValue {
		let collection = Self::get_collection_name(&graphql_type.name);

		println!("{}", collection);

		todo!()
	}

	fn get_operation_name(type_name: &str) -> String {
		format!(
			"get{}",
			pluralizer::pluralize(
				type_name.to_case(convert_case::Case::Pascal).as_str(),
				1,
				false,
			)
		)
	}
}

lazy_static! {
	pub static ref OPERATION_REGISTRY: Mutex<OperationRegistry> = Mutex::new(OperationRegistry::new());
}