pub mod operations;

use juniper::{EmptySubscription, RootNode, EmptyMutation, GraphQLType, GraphQLValue, Registry, ScalarValue, meta::MetaType, GraphQLValueAsync};

use crate::lib::database::api::*;
use crate::lib::database::api::GraphQLType as ApiGraphQLType;

pub type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

pub fn schema(map: GraphQLMap) -> Schema
{
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
		S: ScalarValue
{
	fn name(_: &Self::TypeInfo) -> Option<&str> {
		Some("Query")
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
		where S: 'r
	{
		let mut queries = Vec::new();

		queries.push(registry.field::<QueryField>("getAccount", &info.first().unwrap()));

		registry
			.build_object_type::<Query>(info, &queries)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for Query
	where
		S: ScalarValue
{
	type Context = ();
	type TypeInfo = Vec<ApiGraphQLType>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<S> GraphQLValueAsync<S> for Query
	where
		S: ScalarValue,
		S: Send + Sync
{

}

pub struct QueryField;

impl<S> GraphQLType<S> for QueryField
	where
		S: ScalarValue
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S> where S: 'r {
		let mut fields = Vec::new();

		fields.push(registry.field::<Option<String>>("name", &()));

		registry
			.build_object_type::<QueryField>(info, &fields)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for QueryField
	where
		S: ScalarValue
{
	type Context = ();
	type TypeInfo = ApiGraphQLType;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}