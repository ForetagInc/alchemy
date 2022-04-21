use crate::api::schema::errors::NotFoundError;
use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{
	Arguments, BoxFuture, ExecutionResult, IntoFieldError, Object, Registry, ScalarValue, Value,
};
use rust_arango::ClientError;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::api::schema::operations::get::Get;
use crate::api::schema::operations::get_all::GetAll;
use crate::api::schema::operations::update::Update;
use crate::lib::database::api::{DbEntity, DbRelationship};
use crate::lib::database::aql::{
	AQLFilterOperation, AQLNode, AQLOperation, AQLQuery, AQLQueryBind, AQLQueryParameter,
};

pub mod get;
pub mod get_all;
pub mod update;

type FutureType<'b, S> = BoxFuture<'b, ExecutionResult<S>>;

pub struct OperationRegistry<S>
where
	S: ScalarValue + Send + Sync,
{
	operation_data: HashMap<String, Arc<OperationData<S>>>,
	operations: HashMap<String, OperationEntry<S>>,
}

pub struct OperationEntry<S>
where
	S: ScalarValue + Send + Sync,
{
	pub closure: for<'a> fn(
		&'a OperationData<S>,
		&'a juniper::Arguments<S>,
		AQLQuery<'a>,
	) -> FutureType<'a, S>,
	pub arguments_closure:
		for<'a> fn(&mut Registry<'a, S>, data: &OperationData<S>) -> Vec<Argument<'a, S>>,
	pub field_closure: for<'a> fn(
		&mut Registry<'a, S>,
		name: &str,
		data: &OperationData<S>,
		&OperationRegistry<S>,
	) -> Field<'a, S>,

	pub data: Arc<OperationData<S>>,
}

impl<S> OperationRegistry<S>
where
	S: ScalarValue + Send + Sync,
{
	pub fn new() -> OperationRegistry<S> {
		OperationRegistry {
			operation_data: HashMap::new(),
			operations: HashMap::new(),
		}
	}

	pub fn call_by_key<'b>(
		&'b self,
		key: &str,
		arguments: &'b Arguments<S>,
		query: AQLQuery<'b>,
	) -> Option<FutureType<'b, S>> {
		self.operations
			.get(key)
			.map(|o| (o.closure)(&o.data, arguments, query))
	}

	pub fn get_operations(&self) -> &HashMap<String, OperationEntry<S>> {
		&self.operations
	}

	pub fn get_operation(&self, key: &str) -> Option<&OperationEntry<S>> {
		self.operations.get(key)
	}

	pub fn get_operation_data(&self, key: &str) -> Option<Arc<OperationData<S>>> {
		self.operation_data.get(key).map(|e| e.clone())
	}

	pub fn register_entity(&mut self, entity: Arc<DbEntity>, relationships: Vec<DbRelationship>) {
		let data = Arc::new(OperationData {
			entity: entity.clone(),
			relationships,

			_phantom: Default::default(),
		});

		self.operation_data
			.insert(entity.name.clone(), data.clone());

		vec![
			self.register::<Get>(data.clone()),
			self.register::<GetAll>(data.clone()),
			self.register::<Update>(data.clone()),
		];
	}

	fn register<T: 'static>(&mut self, data: Arc<OperationData<S>>) -> String
	where
		T: Operation<S>,
	{
		let k = T::get_operation_name(&data);

		self.operations.insert(
			k.clone(),
			OperationEntry {
				closure: T::call,
				arguments_closure: T::get_arguments,
				field_closure: T::build_field,
				data,
			},
		);

		k
	}
}

pub struct OperationData<S>
where
	S: ScalarValue,
{
	pub entity: Arc<DbEntity>,
	pub relationships: Vec<DbRelationship>,

	_phantom: PhantomData<S>,
}

pub trait Operation<S>
where
	S: ScalarValue + Send + Sync,
	Self: Send + Sync,
{
	fn call<'b>(
		data: &'b OperationData<S>,
		arguments: &'b Arguments<S>,
		query: AQLQuery<'b>,
	) -> FutureType<'b, S>;

	fn get_operation_name(data: &OperationData<S>) -> String;

	fn get_arguments<'r, 'd>(
		registry: &mut Registry<'r, S>,
		data: &'d OperationData<S>,
	) -> Vec<Argument<'r, S>>;

	fn build_field<'r>(
		registry: &mut Registry<'r, S>,
		name: &str,
		data: &OperationData<S>,
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

fn convert_number<S>(n: &JsonNumber) -> Value<S>
where
	S: ScalarValue + Send + Sync,
{
	return if n.is_i64() {
		let v = n.as_i64().unwrap();

		let res = if v > i32::MAX as i64 {
			i32::MAX
		} else if v < i32::MIN as i64 {
			i32::MIN
		} else {
			v as i32
		};

		Value::scalar(res)
	} else if n.is_u64() {
		let v = n.as_u64().unwrap();

		let res = if v > i32::MAX as u64 {
			i32::MAX
		} else if v < i32::MIN as u64 {
			i32::MIN
		} else {
			v as i32
		};

		Value::scalar(res)
	} else {
		let v = n.as_f64().unwrap();

		Value::scalar(v)
	};
}

fn convert_json_to_juniper_value<S>(data: &JsonMap<String, JsonValue>) -> Value<S>
where
	S: ScalarValue + Send + Sync,
{
	let mut object = Object::<S>::with_capacity(data.len());

	fn convert<S>(val: &JsonValue) -> Value<S>
	where
		S: ScalarValue + Send + Sync,
	{
		match val {
			JsonValue::Null => Value::null(),
			JsonValue::Bool(v) => Value::scalar(v.to_owned()),
			JsonValue::Number(n) => convert_number(n),
			JsonValue::String(s) => Value::scalar(s.to_owned()),
			JsonValue::Array(a) => Value::list(a.iter().map(|i| convert(i)).collect()),
			JsonValue::Object(ref o) => convert_json_to_juniper_value(o),
		}
	}

	for (key, val) in data {
		object.add_field(key, convert(val));
	}

	Value::Object(object)
}

fn get_by_id_filter() -> Box<dyn AQLNode> {
	Box::new(AQLFilterOperation {
		left_node: Box::new(AQLQueryParameter("_key".to_string())),
		operation: AQLOperation::Equal,
		right_node: Box::new(AQLQueryBind("id".to_string())),
	})
}

fn get_single_entry<S>(
	entries: Result<Vec<JsonValue>, ClientError>,
	entity_name: String,
) -> ExecutionResult<S>
where
	S: ScalarValue + Send + Sync,
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
			println!("{:?}", e);

			Err(not_found_error)
		}
	};
}
