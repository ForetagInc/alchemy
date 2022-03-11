pub mod operations;

use juniper::meta::{Field, MetaType};
use juniper::{
	EmptyMutation, EmptySubscription, GraphQLType, GraphQLValue, GraphQLValueAsync, Registry,
	RootNode, ScalarValue,
};

use crate::lib::database::api::GraphQLType as ApiGraphQLType;
use crate::lib::database::api::*;

pub type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

pub fn schema(map: GraphQLMap) -> Schema {
	let mut types: Vec<ApiGraphQLType> = Vec::new();

	for p in map.0 {
		match p {
			GraphQLPrimitive::Type(t) => types.push(*t),
			GraphQLPrimitive::Enum(_) => {}
		}
	}

	RootNode::new_with_info(
		Query,
		EmptyMutation::new(),
		EmptySubscription::new(),
		types,
		(),
		(),
	)
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

		for gql_type in info {
			queries.push(registry.field::<QueryField>("getAccount", gql_type));
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
	type TypeInfo = Vec<ApiGraphQLType>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl GraphQLValueAsync for Query
{}

fn build_field_from_property<'r, S>(
	registry: &mut Registry<'r, S>,
	property: &GraphQLProperty,
	scalar_type: &ScalarType,
) -> Field<'r, S>
	where
		S: ScalarValue,
{
	match scalar_type {
		ScalarType::Array(t) => build_field_from_property(registry, property, &*t),

		ScalarType::Enum(_) if property.required => registry.field::<String>(property.name.as_str(), &()),
		ScalarType::Enum(_) => registry.field::<Option<String>>(property.name.as_str(), &()),

		ScalarType::String if property.required => registry.field::<String>(property.name.as_str(), &()),
		ScalarType::String => registry.field::<Option<String>>(property.name.as_str(), &()),

		ScalarType::Object if property.required => registry.field::<String>(property.name.as_str(), &()),
		ScalarType::Object => registry.field::<Option<String>>(property.name.as_str(), &()),

		ScalarType::Float if property.required => registry.field::<f64>(property.name.as_str(), &()),
		ScalarType::Float => registry.field::<Option<f64>>(property.name.as_str(), &()),

		ScalarType::Int if property.required => registry.field::<i32>(property.name.as_str(), &()),
		ScalarType::Int => registry.field::<Option<i32>>(property.name.as_str(), &()),

		ScalarType::Boolean if property.required => registry.field::<bool>(property.name.as_str(), &()),
		ScalarType::Boolean => registry.field::<Option<bool>>(property.name.as_str(), &()),
	}
}

pub struct QueryField;

impl<S> GraphQLType<S> for QueryField
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
			let field = build_field_from_property(registry, &property, &property.scalar_type);

			fields.push(field);
		}

		registry
			.build_object_type::<QueryField>(info, &fields)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for QueryField
	where
		S: ScalarValue,
{
	type Context = ();
	type TypeInfo = ApiGraphQLType;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}
