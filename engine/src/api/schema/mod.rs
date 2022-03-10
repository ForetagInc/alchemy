pub mod operations;

use juniper::{EmptySubscription, RootNode, EmptyMutation, GraphQLType, GraphQLValue, Registry, ScalarValue, meta::MetaType, GraphQLValueAsync};

use crate::lib::database::api::*;

pub type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

pub fn schema(map: GraphQLMap) -> Schema
{
	let mut types: Vec<crate::lib::database::api::GraphQLType> = Vec::new();

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

		queries.push(registry.field::<Option<i32>>("getAccount", &()));

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
	type TypeInfo = Vec<crate::lib::database::api::GraphQLType>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<S> GraphQLValueAsync<S> for Query
	where
		S: ::juniper::ScalarValue,
		S: Send + Sync
{

}