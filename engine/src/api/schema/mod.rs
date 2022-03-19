pub mod operations;

use crate::api::schema::operations::OperationRegistry;
use juniper::meta::{Field, MetaType};
use juniper::{
	Arguments, BoxFuture, DefaultScalarValue, EmptyMutation, EmptySubscription, ExecutionResult,
	Executor, GraphQLType, GraphQLValue, GraphQLValueAsync, Registry, RootNode, ScalarValue,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::lib::database::api::*;

pub type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

pub fn schema(map: DbMap) -> Schema {
	let mut entities: Vec<Arc<DbEntity>> = Vec::new();
	let mut operation_registry = OperationRegistry::new();

	for p in map.0 {
		match p {
			DbPrimitive::Entity(t) => {
				entities.push(t.clone());

				operation_registry.register_entity(t);
			}
			DbPrimitive::Enum(_) => {}
		}
	}

	RootNode::new_with_info(
		Query,
		EmptyMutation::new(),
		EmptySubscription::new(),
		QueryData {
			entities,
			operation_registry,
		},
		(),
		(),
	)
}

pub struct QueryData {
	entities: Vec<Arc<DbEntity>>,
	operation_registry: OperationRegistry,
}

pub struct Query;

impl<S> GraphQLType<S> for Query
where
	S: ScalarValue,
{
	fn name(_: &Self::TypeInfo) -> Option<&str> {
		Some("Query")
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut queries = Vec::new();

		for entity in &info.entities {
			if let Some(operations) = info
				.operation_registry
				.get_operations_by_entity_name(entity.name.as_str())
			{
				for operation in operations {
					queries.push(registry.field::<QueryField<S>>(operation, &entity));
				}
			}
		}

		registry
			.build_object_type::<Query>(info, &queries)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for Query
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = QueryData;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl GraphQLValueAsync for Query {
	fn resolve_field_async<'b>(
		&'b self,
		info: &'b Self::TypeInfo,
		field_name: &'b str,
		arguments: &'b Arguments<DefaultScalarValue>,
		executor: &'b Executor<Self::Context, DefaultScalarValue>,
	) -> BoxFuture<'b, ExecutionResult<DefaultScalarValue>> {
		info.operation_registry
			.call_by_key(field_name, arguments, executor)
			.unwrap()
	}
}

fn build_field_from_property<'r, S>(
	registry: &mut Registry<'r, S>,
	property: &DbProperty,
	scalar_type: &DbScalarType,
	enforce_required: bool,
) -> Field<'r, S>
where
	S: ScalarValue,
{
	fn build_field<'r, T, S>(
		registry: &mut Registry<'r, S>,
		property: &DbProperty,
		required: bool,
	) -> Field<'r, S>
	where
		S: ScalarValue + 'r,
		T: GraphQLType<S, Context = (), TypeInfo = ()>,
	{
		let is_array = matches!(property.scalar_type, DbScalarType::Array(_));

		if required && !is_array {
			registry.field::<T>(property.name.as_str(), &())
		} else {
			registry.field::<Option<T>>(property.name.as_str(), &())
		}
	}

	match scalar_type {
		DbScalarType::Array(t) => {
			let mut field = build_field_from_property(registry, property, &t, false);

			if property.required && enforce_required {
				field.field_type = juniper::Type::NonNullList(Box::new(field.field_type));
			} else {
				field.field_type = juniper::Type::List(Box::new(field.field_type));
			}

			field
		}

		DbScalarType::Enum(_) => build_field::<String, S>(registry, property, property.required),
		DbScalarType::String => build_field::<String, S>(registry, property, property.required),
		DbScalarType::Object => build_field::<String, S>(registry, property, property.required),
		DbScalarType::Float => build_field::<f64, S>(registry, property, property.required),
		DbScalarType::Int => build_field::<i32, S>(registry, property, property.required),
		DbScalarType::Boolean => build_field::<bool, S>(registry, property, property.required),
	}
}

pub struct QueryField<S> {
	properties: HashMap<String, Box<dyn GraphQLValue<S, Context = (), TypeInfo = ()>>>,
}

impl<S> GraphQLType<S> for QueryField<S>
where
	S: ScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut fields = Vec::new();

		for property in &info.properties {
			let field = build_field_from_property(registry, &property, &property.scalar_type, true);

			fields.push(field);
		}

		registry
			.build_object_type::<QueryField<S>>(info, &fields)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for QueryField<S>
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = DbEntity;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}

	fn resolve_field(
		&self,
		_info: &Self::TypeInfo,
		field_name: &str,
		_arguments: &Arguments<S>,
		executor: &Executor<Self::Context, S>,
	) -> ExecutionResult<S> {
		let value = self.properties.get(field_name).unwrap();

		executor.resolve(&(), &value)
	}
}
