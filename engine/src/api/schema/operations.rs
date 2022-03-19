use convert_case::Casing;
use juniper::{Arguments, BoxFuture, ExecutionResult, Executor, ID, Registry, ScalarValue};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use juniper::meta::Argument;
use crate::api::schema::fields::QueryField;

use crate::lib::database::api::DbEntity;

type FutureType<'b, S> = BoxFuture<'b, ExecutionResult<S>>;

pub struct OperationRegistry<S>
	where
		S: ScalarValue + Send + Sync
{
	operations: HashMap<String, Box<dyn Operation<S>>>,
}

impl<S> OperationRegistry<S>
	where
		S: ScalarValue + Send + Sync
{
	pub fn new() -> OperationRegistry<S> {
		OperationRegistry {
			operations: HashMap::new(),
		}
	}

	pub fn call_by_key<'b>(
		&self,
		key: &str,
		arguments: &Arguments<S>,
		executor: &'b Executor<(), S>,
	) -> Option<FutureType<'b, S>> {
		self.operations
			.get(key)
			.map(|o| o.call(arguments, executor))
	}

	pub fn get_operations(&self) -> &HashMap<String, Box<dyn Operation<S>>> {
		&self.operations
	}

	pub fn register_entity(&mut self, entity: Arc<DbEntity>) {
		vec![
			self.register(Get(entity.clone(), PhantomData::default()))
		];
	}

	fn register<T: 'static>(&mut self, operation: T) -> String
	where
		T: Operation<S>,
	{
		let k = operation.get_operation_name();

		self.operations.insert(k.clone(), Box::new(operation));

		k
	}
}

pub trait Operation<S>
	where
		S: ScalarValue,
		Self: Send + Sync
{
	fn call<'b>(
		&self,
		arguments: &Arguments<S>,
		executor: &'b Executor<(), S>,
	) -> FutureType<'b, S>;

	fn get_operation_name(&self) -> String;

	fn get_entity(&self) -> Arc<DbEntity>;

	fn get_arguments<'r>(&self, registry: &mut Registry<'r, S>) -> Vec<Argument<'r, S>>;

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

pub struct Get<S>(Arc<DbEntity>, PhantomData<S>);

impl<S> Operation<S> for Get<S>
	where
		S: ScalarValue + Send + Sync
{
	fn call<'b>(
		&self,
		arguments: &Arguments<S>,
		executor: &'b Executor<(), S>,
	) -> FutureType<'b, S> {
		let collection = Self::get_collection_name(&self.0.name);

		let entity = self.0.clone();

		Box::pin(async move {
			println!("{}", collection);

			let mut properties: HashMap<
				String,
				Box<dyn juniper::GraphQLValue<S, Context = (), TypeInfo = ()>>,
			> = HashMap::new();

			properties.insert("firstName".to_string(), Box::new("Kenneth"));
			properties.insert("lastName".to_string(), Box::new("Gomez"));

			executor.resolve::<QueryField<S>>(&entity, &QueryField { properties })
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

	fn get_arguments<'r>(&self, registry: &mut Registry<'r, S>) -> Vec<Argument<'r, S>> {
		vec![
			registry.arg::<ID>("surname", &())
		]
	}
}
