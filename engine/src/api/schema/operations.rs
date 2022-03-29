use crate::api::schema::errors::NotFoundError;
use convert_case::Casing;
use juniper::meta::Argument;
use juniper::{
	graphql_value, Arguments, BoxFuture, ExecutionResult, Executor, FieldError, GraphQLValue,
	IntoFieldError, Registry, ScalarValue, Value, ID,
};
use rust_arango::{AqlQuery, ClientError};
use serde_json::Value as JsonValue;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::api::schema::fields::QueryField;
use crate::lib::database::api::{DbEntity, DbRelationship, DbScalarType};
use crate::lib::database::DATABASE;

type FutureType<'b, S> = BoxFuture<'b, ExecutionResult<S>>;

pub struct OperationRegistry<S>
where
	S: ScalarValue + Send + Sync,
{
	operations: HashMap<String, Box<dyn Operation<S>>>,
}

impl<S> OperationRegistry<S>
where
	S: ScalarValue + Send + Sync,
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

	pub fn register_entity(
		&mut self,
		entity: Arc<DbEntity>,
		relationships: Arc<Vec<DbRelationship>>,
	) {
		vec![self.register(Get(
			Arc::new(OperationData {
				entity: entity.clone(),
				relationships: relationships.clone(),
			}),
			PhantomData::default(),
		))];
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
	Self: Send + Sync,
{
	fn call<'b>(
		&'b self,
		arguments: &'b Arguments<S>,
		executor: &'b Executor<(), S>,
	) -> FutureType<'b, S>;

	fn get_operation_name(&self) -> String;

	fn get_entity(&self) -> Arc<DbEntity>;

	fn get_relationships(&self) -> Arc<Vec<DbRelationship>>;

	fn get_data(&self) -> Arc<OperationData>;

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

	fn get_relationship_edge_name(relationship: &DbRelationship) -> String
	where
		Self: Sized,
	{
		format!(
			"{}_{}",
			pluralizer::pluralize(
				relationship
					.from
					.name
					.to_case(convert_case::Case::Snake)
					.as_str(),
				1,
				false,
			),
			relationship.name
		)
	}
}

fn map_value_to_type<S>(
	value: &JsonValue,
	db_type: &DbScalarType,
) -> Box<dyn GraphQLValue<S, Context = (), TypeInfo = ()> + Send>
where
	S: ScalarValue,
{
	match db_type {
		DbScalarType::Enum(_) | DbScalarType::String => {
			Box::new(value.as_str().map(|s| s.to_string()))
		}
		DbScalarType::Array(v) => {
			let content = match value.as_array() {
				Some(arr) => {
					let mut values = Vec::new();

					for item in arr {
						values.push(map_value_to_type::<S>(item, &*v))
					}

					Some(values)
				}
				None => None,
			};

			Box::new(content)
		}
		DbScalarType::Object => Box::new("object".to_string()),
		DbScalarType::Float => Box::new(value.as_f64()),
		DbScalarType::Int => Box::new(value.as_i64().map(|v| v as i32)),
		DbScalarType::Boolean => Box::new(value.as_bool()),
	}
}

pub struct OperationData {
	pub entity: Arc<DbEntity>,
	pub relationships: Arc<Vec<DbRelationship>>,
}

pub struct Get<S>(Arc<OperationData>, PhantomData<S>);

impl<S> Operation<S> for Get<S>
where
	S: ScalarValue + Send + Sync,
{
	fn call<'b>(
		&'b self,
		arguments: &'b Arguments<S>,
		executor: &'b Executor<(), S>,
	) -> FutureType<'b, S> {
		let entity = self.get_entity();

		let collection = Self::get_collection_name(&entity.name);

		Box::pin(async move {
			println!("{} {:?}", collection.clone(), arguments);

			let mut properties: HashMap<
				String,
				Box<dyn juniper::GraphQLValue<S, Context = (), TypeInfo = ()> + Send>,
			> = HashMap::new();

			let mut relationship_aql = String::new();

			for (i, _) in self.get_relationships().iter().enumerate() {
				relationship_aql.push_str(
					format!(
						"
						{{
							name: @relationship_{id},
							values: (FOR b_{id} IN OUTBOUND a @@relationship_{id} RETURN b_{id})
						}},",
						id = i
					)
					.as_str(),
				)
			}

			let query = format!(
				"
					FOR a IN @@collection
						FILTER a.`_key` == @key
							LIMIT 1
					LET rel = (
						[
							{}
						]
					)
					RETURN {{
						entity: a,
						relationships: rel
					}}",
				relationship_aql
			);

			let mut entries_query = AqlQuery::builder()
				.query(query.as_str())
				.bind_var("@collection", collection.clone())
				.bind_var("key", arguments.get::<String>("id").unwrap());

			for (i, relationship) in self.get_relationships().iter().enumerate() {
				let relationship_name = Self::get_relationship_edge_name(relationship);

				fn string_to_static_str<'b>(s: String) -> &'b str {
					Box::leak(s.into_boxed_str())
				}

				entries_query = entries_query
					.bind_var(
						string_to_static_str::<'b>(format!("@relationship_{}", i)),
						relationship_name.as_str(),
					)
					.bind_var(
						string_to_static_str::<'b>(format!("relationship_{}", i)),
						relationship.name.as_str(),
					)
			}

			let entries: Result<Vec<JsonValue>, ClientError> = DATABASE
				.get()
				.await
				.database
				.aql_query(entries_query.build())
				.await;

			let not_found_error = NotFoundError::new(entity.name.clone()).into_field_error();

			return match entries {
				Ok(data) => {
					if let Some(first) = data.first() {
						for relationship in first["relationships"].as_array().unwrap() {
							let mut rel_properties: HashMap<
								String,
								Box<
									dyn juniper::GraphQLValue<S, Context = (), TypeInfo = ()>
										+ Send,
								>,
							> = HashMap::new();

							let relationship_name = relationship["name"].as_str().unwrap();
							let values = relationship["values"].as_array().unwrap();

							properties
								.insert(relationship_name.to_string(), Box::new("asd".to_string()));
						}

						for property in &entity.properties {
							let prop_name = property.name.as_str();

							properties.insert(
								prop_name.to_string(),
								map_value_to_type::<S>(
									&first["entity"][prop_name],
									&property.scalar_type,
								),
							);
						}

						return executor.resolve::<QueryField<S>>(
							&self.get_data(),
							&QueryField { properties },
						);
					}

					Err(not_found_error)
				}
				Err(e) => {
					println!("{:?}", e);

					Err(not_found_error)
				}
			};
		})
	}

	fn get_operation_name(&self) -> String {
		format!(
			"get{}",
			pluralizer::pluralize(
				self.get_entity()
					.name
					.to_case(convert_case::Case::Pascal)
					.as_str(),
				1,
				false,
			)
		)
	}

	fn get_entity(&self) -> Arc<DbEntity> {
		self.0.entity.clone()
	}

	fn get_relationships(&self) -> Arc<Vec<DbRelationship>> {
		self.0.relationships.clone()
	}

	fn get_data(&self) -> Arc<OperationData> {
		self.0.clone()
	}

	fn get_arguments<'r>(&self, registry: &mut Registry<'r, S>) -> Vec<Argument<'r, S>> {
		vec![registry.arg::<ID>("id", &())]
	}
}
