pub mod enums;
pub mod errors;
pub mod fields;
pub mod input;
pub mod operations;
pub mod scalars;
mod utils;

use crate::api::schema::enums::{DbEnumInfo, GraphQLEnum};
use crate::api::schema::fields::SchemaFieldFactory;
use crate::api::schema::operations::OperationRegistry;
use juniper::meta::{Argument, MetaType};
use juniper::{
	Arguments, BoxFuture, EmptySubscription, ExecutionResult, Executor, FromInputValue,
	GraphQLType, GraphQLValue, GraphQLValueAsync, InputValue, Registry, RootNode, ScalarValue,
};
use std::sync::Arc;

use crate::lib::database::api::*;

pub type Schema = RootNode<'static, SchemaType, SchemaType, EmptySubscription>;

pub trait AsyncScalarValue = ScalarValue + Send + Sync;

pub fn schema(map: DbMap) -> Schema {
	let mut operation_registry = OperationRegistry::new();

	for p in map.primitives {
		match p {
			DbPrimitive::Entity(t) => {
				let mut relationships = Vec::new();

				for relationship in &map.relationship_fields {
					if relationship.from.name == t.name {
						relationships.push(relationship.clone())
					}
				}

				operation_registry.register_entity(t, relationships);
			}
			DbPrimitive::Enum(_) => {}
		}
	}

	let relationship_fields = Arc::new(map.relationship_fields.clone());
	let query_info = SchemaData {
		operation_registry: Arc::new(operation_registry),
		relationship_fields,
		kind: SchemaKind::Query,
	};

	let mutation_info = SchemaData {
		kind: SchemaKind::Mutation,
		..query_info.clone()
	};

	RootNode::new_with_info(
		SchemaType,
		SchemaType,
		EmptySubscription::new(),
		query_info,
		mutation_info,
		(),
	)
}

#[derive(PartialEq, Clone)]
pub enum SchemaKind {
	Query,
	Mutation,
}

#[derive(Clone)]
pub struct SchemaData<S>
where
	S: AsyncScalarValue,
{
	kind: SchemaKind,
	operation_registry: Arc<OperationRegistry<S>>,
	relationship_fields: Arc<Vec<DbRelationshipField>>,
}

pub struct SchemaType;

impl<S> GraphQLType<S> for SchemaType
where
	S: AsyncScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(match info.kind {
			SchemaKind::Query => "Query",
			SchemaKind::Mutation => "Mutation",
		})
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut queries = Vec::new();

		for (name, operation) in info.operation_registry.get_operations(info.kind.clone()) {
			queries.push(SchemaFieldFactory::new(
				name,
				operation,
				registry,
				&info.operation_registry,
			));
		}

		registry
			.build_object_type::<SchemaType>(info, &queries)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for SchemaType
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = SchemaData<S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<S> GraphQLValueAsync<S> for SchemaType
where
	S: AsyncScalarValue,
{
	fn resolve_field_async<'b>(
		&'b self,
		info: &'b Self::TypeInfo,
		field_name: &'b str,
		arguments: &'b Arguments<S>,
		executor: &'b Executor<Self::Context, S>,
	) -> BoxFuture<'b, ExecutionResult<S>> {
		Box::pin(async move {
			executor
				.resolve_async(
					info,
					&SchemaFieldFactory::new_resolver(field_name, arguments),
				)
				.await
		})
	}
}

pub fn input_value_to_string<S>(data: &InputValue<S>) -> String
where
	S: ScalarValue,
{
	serde_json::to_string(data).unwrap()
}

pub fn build_argument_from_property<'r, S>(
	registry: &mut Registry<'r, S>,
	property: &DbProperty,
	scalar_type: &DbScalarType,
	required: bool,
) -> Argument<'r, S>
where
	S: ScalarValue,
{
	fn build_argument<'r, T, S>(
		registry: &mut Registry<'r, S>,
		property: &DbProperty,
		required: bool,
		info: &T::TypeInfo,
	) -> Argument<'r, S>
	where
		S: ScalarValue + 'r,
		T: GraphQLType<S, Context = ()> + FromInputValue<S>,
	{
		if required {
			registry.arg::<T>(property.name.as_str(), info)
		} else {
			registry.arg::<Option<T>>(property.name.as_str(), info)
		}
	}

	match scalar_type {
		DbScalarType::Array(t) => {
			let mut argument = build_argument_from_property(registry, property, &t, required);

			if required {
				argument.arg_type = juniper::Type::NonNullList(Box::new(argument.arg_type));
			} else {
				argument.arg_type = juniper::Type::List(Box::new(argument.arg_type));
			}

			argument
		}

		DbScalarType::Enum(values) => build_argument::<GraphQLEnum, S>(
			registry,
			property,
			required,
			&DbEnumInfo {
				name: property.associated_type.clone().unwrap(),
				properties: values.clone(),
			},
		),
		DbScalarType::String => build_argument::<String, S>(registry, property, required, &()),
		DbScalarType::Object => build_argument::<String, S>(registry, property, required, &()),
		DbScalarType::Float => build_argument::<f64, S>(registry, property, required, &()),
		DbScalarType::Int => build_argument::<i32, S>(registry, property, required, &()),
		DbScalarType::Boolean => build_argument::<bool, S>(registry, property, required, &()),
	}
}
