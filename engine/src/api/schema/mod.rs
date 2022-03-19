pub mod operations;
pub mod fields;

use crate::api::schema::operations::OperationRegistry;
use juniper::meta::MetaType;
use juniper::{
	Arguments, BoxFuture, DefaultScalarValue, EmptyMutation, EmptySubscription, ExecutionResult,
	Executor, GraphQLType, GraphQLValue, GraphQLValueAsync, Registry, RootNode, ScalarValue,
};
use crate::api::schema::fields::QueryFieldFactory;

use crate::lib::database::api::*;

pub type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

pub fn schema(map: DbMap) -> Schema {
	let mut operation_registry = OperationRegistry::new();

	for p in map.0 {
		match p {
			DbPrimitive::Entity(t) => {
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
			operation_registry,
		},
		(),
		(),
	)
}

pub struct QueryData {
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

		for (name, operation) in info.operation_registry.get_operations() {
			queries.push(QueryFieldFactory::new(name, &operation, registry));
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