use convert_case::Casing;
use juniper::{Arguments, BoxFuture, DefaultScalarValue, ExecutionResult, Executor};
use std::collections::HashMap;
use std::sync::Arc;
use crate::api::schema::fields::QueryField;

use crate::lib::database::api::DbEntity;

type FutureType<'b> = BoxFuture<'b, ExecutionResult<DefaultScalarValue>>;

pub struct OperationRegistry {
	operations: HashMap<String, Box<dyn Operation>>,
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

	pub fn get_operations(&self) -> &HashMap<String, Box<dyn Operation>> {
		&self.operations
	}

	pub fn register_entity(&mut self, entity: Arc<DbEntity>) {
		vec![self.register(Get(entity.clone()))];
	}

	fn register<T: 'static>(&mut self, operation: T) -> String
	where
		T: Operation,
	{
		let k = operation.get_operation_name();

		self.operations.insert(k.clone(), Box::new(operation));

		k
	}
}

pub trait Operation
	where Self: Send + Sync
{
	fn call<'b>(
		&self,
		arguments: &Arguments<DefaultScalarValue>,
		executor: &'b Executor<(), DefaultScalarValue>,
	) -> FutureType<'b>;

	fn get_operation_name(&self) -> String;

	fn get_entity(&self) -> Arc<DbEntity>;

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

pub struct Get(Arc<DbEntity>);

impl Operation for Get {
	fn call<'b>(
		&self,
		arguments: &Arguments<DefaultScalarValue>,
		executor: &'b Executor<(), DefaultScalarValue>,
	) -> FutureType<'b> {
		let collection = Self::get_collection_name(&self.0.name);

		let entity = self.0.clone();

		Box::pin(async move {
			println!("{}", collection);

			let mut properties: HashMap<
				String,
				Box<dyn juniper::GraphQLValue<DefaultScalarValue, Context = (), TypeInfo = ()>>,
			> = HashMap::new();

			properties.insert("firstName".to_string(), Box::new("Kenneth"));
			properties.insert("lastName".to_string(), Box::new("Gomez"));

			executor.resolve::<QueryField<DefaultScalarValue>>(&entity, &QueryField { properties })
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

	fn get_entity(&self) -> Arc<DbEntity> {
		self.0.clone()
	}
}
