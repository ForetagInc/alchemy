use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, BoxFuture, ExecutionResult, InputValue, IntoFieldError, Registry, Value};
use rust_arango::{AqlQuery, ClientError};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use crate::api::schema::errors::{DatabaseError, NotFoundError};
use crate::api::schema::utils::convert_json_to_juniper_value;
use crate::api::schema::{AsyncScalarValue, SchemaKind};
use crate::lib::database::api::{DbEntity, DbRelationship};
use crate::lib::database::aql::{
	AQLFilterOperation, AQLLogicalFilter, AQLLogicalOperator, AQLNode, AQLOperation, AQLQuery,
	AQLQueryBind, AQLQueryParameter,
};
use crate::lib::database::DATABASE;

pub mod utils;

pub mod create;
pub mod get;
pub mod get_all;
pub mod remove;
pub mod remove_all;
pub mod update;
pub mod update_all;

type FutureType<'b, S> = BoxFuture<'b, ExecutionResult<S>>;

pub struct OperationRegistry<S>
where
	S: AsyncScalarValue,
{
	operation_data: HashMap<String, Arc<OperationData>>,
	operations: HashMap<String, OperationEntry<S>>,
}

pub struct OperationEntry<S>
where
	S: AsyncScalarValue,
{
	pub closure: for<'a> fn(&'a OperationData, &'a Arguments<S>, AQLQuery) -> FutureType<'a, S>,
	pub arguments_closure: for<'a> fn(
		&mut Registry<'a, S>,
		data: &OperationData,
		&OperationRegistry<S>,
	) -> Vec<Argument<'a, S>>,
	pub field_closure: for<'a> fn(
		&mut Registry<'a, S>,
		name: &str,
		data: &OperationData,
		&OperationRegistry<S>,
	) -> Field<'a, S>,

	pub data: Arc<OperationData>,
	pub kind: SchemaKind,
}

impl<S> OperationRegistry<S>
where
	S: AsyncScalarValue,
{
	pub fn new() -> OperationRegistry<S> {
		OperationRegistry {
			operation_data: HashMap::new(),
			operations: HashMap::new(),
		}
	}

	pub fn get_operations(&self, kind: SchemaKind) -> HashMap<&String, &OperationEntry<S>> {
		self.operations
			.iter()
			.filter(|(_, entry)| entry.kind == kind)
			.collect()
	}

	pub fn get_operation(&self, key: &str) -> Option<&OperationEntry<S>> {
		self.operations.get(key)
	}

	pub fn get_operation_data(&self, key: &str) -> Option<Arc<OperationData>> {
		self.operation_data.get(key).map(|e| e.clone())
	}

	pub fn register_entity(&mut self, entity: Arc<DbEntity>, relationships: Vec<DbRelationship>) {
		let data = Arc::new(OperationData {
			entity: entity.clone(),
			relationships,
		});

		self.operation_data
			.insert(entity.name.clone(), data.clone());

		vec![
			self.register::<get::Get>(data.clone(), SchemaKind::Query),
			self.register::<get_all::GetAll>(data.clone(), SchemaKind::Query),
			self.register::<update::Update>(data.clone(), SchemaKind::Mutation),
			self.register::<update_all::UpdateAll>(data.clone(), SchemaKind::Mutation),
			self.register::<remove::Remove>(data.clone(), SchemaKind::Mutation),
			self.register::<remove_all::RemoveAll>(data.clone(), SchemaKind::Mutation),
			self.register::<create::Create>(data.clone(), SchemaKind::Mutation),
		];
	}

	fn register<T: 'static>(&mut self, data: Arc<OperationData>, kind: SchemaKind) -> String
	where
		T: Operation<S>,
	{
		let k = T::get_operation_name(&data);

		if self.operations.contains_key(&k) {
			println!("warn: registering duplicated operation `{}`", &k)
		}

		self.operations.insert(
			k.clone(),
			OperationEntry {
				closure: T::call,
				arguments_closure: T::get_arguments,
				field_closure: T::build_field,
				data,
				kind,
			},
		);

		k
	}
}

pub struct OperationData {
	pub entity: Arc<DbEntity>,
	pub relationships: Vec<DbRelationship>,
}

pub trait Operation<S>
where
	S: AsyncScalarValue,
	Self: Send + Sync,
{
	fn call<'b>(
		data: &'b OperationData,
		arguments: &'b Arguments<S>,
		query: AQLQuery,
	) -> FutureType<'b, S>;

	fn get_operation_name(data: &OperationData) -> String;

	fn get_arguments<'r, 'd>(
		registry: &mut Registry<'r, S>,
		data: &'d OperationData,
		operation_registry: &OperationRegistry<S>,
	) -> Vec<Argument<'r, S>>;

	fn build_field<'r>(
		registry: &mut Registry<'r, S>,
		name: &str,
		data: &OperationData,
		operation_registry: &OperationRegistry<S>,
	) -> Field<'r, S>;

	fn get_relationship_edge_name(relationship: &DbRelationship) -> String {
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

fn get_filter_by_indices_attributes<S>(
	attributes: &HashMap<String, InputValue<S>>,
) -> Box<dyn AQLNode>
where
	S: AsyncScalarValue,
{
	let mut nodes: Vec<Box<dyn AQLNode>> = Vec::new();

	for (key, _) in attributes {
		nodes.push(Box::new(AQLFilterOperation {
			left_node: Box::new(AQLQueryParameter(key.clone())),
			operation: AQLOperation::Equal,
			right_node: Box::new(AQLQueryBind(key.clone())),
		}));
	}

	Box::new(AQLLogicalFilter {
		operation: AQLLogicalOperator::AND,
		nodes,
	})
}

fn get_filter_by_key() -> Box<dyn AQLNode> {
	Box::new(AQLFilterOperation {
		left_node: Box::new(AQLQueryParameter("_key".to_string())),
		operation: AQLOperation::Equal,
		right_node: Box::new(AQLQueryBind("_key".to_string())),
	})
}

fn get_filter_in_keys() -> Box<dyn AQLNode> {
	Box::new(AQLFilterOperation {
		left_node: Box::new(AQLQueryParameter("_key".to_string())),
		operation: AQLOperation::In,
		right_node: Box::new(AQLQueryBind("_keys".to_string())),
	})
}

fn get_single_entry<S>(
	entries: Result<Vec<JsonValue>, ClientError>,
	entity_name: String,
) -> ExecutionResult<S>
where
	S: AsyncScalarValue,
{
	let not_found_error = NotFoundError::new(entity_name).into_field_error();

	return match entries {
		Ok(data) => {
			if let Some(first) = data.first() {
				let time = std::time::Instant::now();

				let ret = Ok(convert_json_to_juniper_value(first.as_object().unwrap()));

				println!("Conversion: {:?}", time.elapsed());

				return ret;
			}

			Err(not_found_error)
		}
		Err(e) => {
			let message = format!("{}", e);

			Err(DatabaseError::new(message).into_field_error())
		}
	};
}

fn get_multiple_entries<S>(entries: Result<Vec<JsonValue>, ClientError>) -> ExecutionResult<S>
where
	S: AsyncScalarValue,
{
	return match entries {
		Ok(data) => {
			let mut output = Vec::<Value<S>>::new();

			let time = std::time::Instant::now();

			for datum in data {
				output.push(convert_json_to_juniper_value(datum.as_object().unwrap()));
			}

			println!("Output conversion: {:?}", time.elapsed());

			Ok(Value::list(output))
		}
		Err(e) => {
			let message = format!("{}", e);

			Err(DatabaseError::new(message).into_field_error())
		}
	};
}

pub enum QueryReturnType {
	Single,
	Multiple,
}

async fn execute_internal_query<S>(
	query: AQLQuery,
	collection: &str,
	query_arguments: HashMap<String, InputValue<S>>,
	query_hardcoded_arguments: HashMap<String, InputValue<S>>,
) -> Vec<JsonValue>
where
	S: AsyncScalarValue,
{
	let time = std::time::Instant::now();

	let aql = query.to_aql();

	println!("Internal Query: {}", &aql);

	let mut entries_query = AqlQuery::builder()
		.query(&aql)
		.bind_var("@collection".to_string(), collection.clone());

	utils::assign_parameters!(query_arguments, (key, v) -> {
		entries_query = entries_query.bind_var(query.get_argument_key(key.as_str()), v);
	});

	utils::assign_parameters!(query_hardcoded_arguments, (key, v) -> {
		entries_query = entries_query.bind_var(key.as_str(), v);
	});

	let entries: Result<Vec<JsonValue>, ClientError> = DATABASE
		.get()
		.await
		.database
		.aql_query(entries_query.build())
		.await;

	println!("Internal Query AQL: {:?}", time.elapsed());

	entries.unwrap()
}

async fn execute_query<'a, S, T>(
	query: AQLQuery,
	entity: &'a DbEntity,
	collection: &'a str,
	return_type: QueryReturnType,
	query_arguments: HashMap<String, InputValue<S>>,
	raw_arguments: HashMap<String, T>,
) -> ExecutionResult<S>
where
	S: AsyncScalarValue,
	T: Into<JsonValue>,
{
	// TODO: Change HashMap InputValue to be parsed before

	let time = std::time::Instant::now();

	let query_str = query.to_aql();

	println!("{}", &query_str);

	let mut entries_query = AqlQuery::builder()
		.query(&query_str)
		.bind_var("@collection".to_string(), collection.clone());

	utils::assign_parameters!(query_arguments, (key, v) -> {
		entries_query = entries_query.bind_var(query.get_argument_key(key.as_str()), v);
	});

	for (k, v) in raw_arguments {
		entries_query = entries_query.bind_var(query.get_argument_key(k.as_str()), v);
	}

	let entries: Result<Vec<JsonValue>, ClientError> = DATABASE
		.get()
		.await
		.database
		.aql_query(entries_query.build())
		.await;

	println!("SQL: {:?}", time.elapsed());

	match return_type {
		QueryReturnType::Single => get_single_entry(entries, entity.name.clone()),
		QueryReturnType::Multiple => get_multiple_entries(entries),
	}
}
