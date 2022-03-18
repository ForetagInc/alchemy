use convert_case::Casing;
use juniper::{Arguments, BoxFuture, DefaultScalarValue, ExecutionResult, Executor};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::lib::database::api::DbEntity;

type FutureType<'b> = BoxFuture<'b, ExecutionResult<DefaultScalarValue>>;

pub struct OperationRegistry {
	operations: HashMap<String, Box<dyn Operation + Send>>,
}

impl OperationRegistry {
	pub fn new() -> OperationRegistry {
		OperationRegistry {
			operations: HashMap::new(),
		}
	}

	pub fn call_by_key<'b>(
		&self,
		key: &str,
		arguments: &Arguments<DefaultScalarValue>,
		executor: &'b Executor<(), DefaultScalarValue>,
	) -> Option<FutureType<'b>> {
		self.operations
			.get(key)
			.map(|o| o.call(arguments, executor))
	}

	pub fn register_entity(&mut self, graphql_type: DbEntity) -> Vec<Option<String>> {
		vec![
			self.register(Get(graphql_type))
		]
	}

	fn register<T: 'static>(&mut self, operation: T) -> Option<String>
	where
		T: Operation + Send,
	{
		let k = operation.get_operation_name();

		self.operations
			.insert(k.clone(), Box::new(operation))
			.map(|_| k)
	}
}

pub trait Operation {
	fn call<'b>(
		&self,
		arguments: &Arguments<DefaultScalarValue>,
		executor: &'b Executor<(), DefaultScalarValue>,
	) -> FutureType<'b>;

	fn get_operation_name(&self) -> String;

	fn get_collection_name(type_name: &str) -> String
	where
		Self: Sized,
	{
		pluralizer::pluralize(
			type_name.to_case(convert_case::Case::Snake).as_str(),
			2,
			false,
		)
	}
}

pub struct Get(DbEntity);

impl Operation for Get {
	fn call<'b>(
		&self,
		arguments: &Arguments<DefaultScalarValue>,
		executor: &'b Executor<(), DefaultScalarValue>,
	) -> FutureType<'b> {
		let collection = Self::get_collection_name(&self.0.name);

		Box::pin(async move {
			println!("{}", collection);

			executor.resolve_with_ctx(&(), &"a")
		})
	}

	fn get_operation_name(&self) -> String {
		format!(
			"get{}",
			pluralizer::pluralize(
				self.0.name.to_case(convert_case::Case::Pascal).as_str(),
				1,
				false,
			)
		)
	}
}

lazy_static! {
	pub static ref OPERATION_REGISTRY: Mutex<OperationRegistry> =
		Mutex::new(OperationRegistry::new());
}
