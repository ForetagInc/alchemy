use std::any::Any;
use convert_case::Casing;
use juniper::{Arguments, BoxFuture, ExecutionResult, Executor, FieldError, graphql_value, GraphQLValue, ID, IntoFieldError, Registry, ScalarValue, Value};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use juniper::meta::Argument;
use rust_arango::{AqlQuery, ClientError};
use serde_json::Value as JsonValue;
use crate::api::schema::errors::NotFoundError;

use crate::api::schema::fields::QueryField;
use crate::lib::database::api::{DbEntity, DbScalarType};
use crate::lib::database::DATABASE;

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
		&'b self,
		key: &str,
		arguments: &'b Arguments<S>,
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
		&'b self,
		arguments: &'b Arguments<S>,
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

fn map_value_to_type<S>(value: &JsonValue, db_type: &DbScalarType) -> Box<dyn GraphQLValue<S, Context = (), TypeInfo = ()> + Send>
	where
		S: ScalarValue
{
	match db_type {
		DbScalarType::Array(_) => {
			Box::new("array".to_string())
		}
		DbScalarType::Enum(_) => {
			Box::new("enum".to_string())
		}
		DbScalarType::String => {
			Box::new(value.as_str().map(|s| s.to_string()))
		}
		DbScalarType::Object => {
			Box::new("object".to_string())
		}
		DbScalarType::Float => {
			Box::new(value.as_f64())
		}
		DbScalarType::Int => {
			Box::new(value.as_i64().map(|v| v as i32))
		}
		DbScalarType::Boolean => {
			Box::new(value.as_bool())
		}
	}
}

pub struct Get<S>(Arc<DbEntity>, PhantomData<S>);

impl<S> Operation<S> for Get<S>
	where
		S: ScalarValue + Send + Sync
{
	fn call<'b>(
		&'b self,
		arguments: &'b Arguments<S>,
		executor: &'b Executor<(), S>,
	) -> FutureType<'b, S> {
		let collection = Self::get_collection_name(&self.0.name);

		let entity = self.0.clone();

		Box::pin(async move {
			println!("{} {:?}", collection, arguments);

			let mut properties: HashMap<
				String,
				Box<dyn juniper::GraphQLValue<S, Context = (), TypeInfo = ()> + Send>,
			> = HashMap::new();

			let entries_query = AqlQuery::builder()
				.query("FOR a IN @@collection
						FILTER a.`_key` == @key
						LIMIT 1
						RETURN a")
				.bind_var("@collection", collection)
				.bind_var("key", arguments.get::<String>("id").unwrap())
				.build();

			let entries: Result<Vec<JsonValue>, ClientError> = DATABASE
				.get()
				.await
				.database
				.aql_query(entries_query)
				.await;

			let not_found_error = NotFoundError::new(self.0.name.clone()).into_field_error();

			return match entries {
				Ok(data) => {
					if let Some(first) = data.first() {
						for property in &self.0.properties {
							let prop_name = property.name.as_str();

							properties.insert(
								prop_name.to_string(),
								map_value_to_type::<S>(&first[prop_name], &property.scalar_type)
							);
						}

						return executor.resolve::<QueryField<S>>(&entity, &QueryField { properties })
					}

					Err(not_found_error)
				}
				Err(e) => {
					println!("{:?}", e);

					Err(not_found_error)
				}
			}
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
			registry.arg::<ID>("id", &())
		]
	}
}
