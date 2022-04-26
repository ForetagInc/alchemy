pub mod enums;
pub mod errors;
pub mod fields;
pub mod operations;

use crate::api::schema::fields::SchemaFieldFactory;
use crate::api::schema::operations::OperationRegistry;
use juniper::meta::MetaType;
use juniper::{
	Arguments, BoxFuture, EmptySubscription, ExecutionResult, Executor, GraphQLType,
	GraphQLValue, GraphQLValueAsync, Registry, RootNode, ScalarValue,
};
use std::sync::Arc;

use crate::lib::database::api::*;

pub type Schema = RootNode<'static, SchemaType, SchemaType, EmptySubscription>;

pub fn schema(map: DbMap) -> Schema {
	let mut operation_registry = OperationRegistry::new();

	for p in map.primitives {
		match p {
			DbPrimitive::Entity(t) => {
				let mut relationships = Vec::new();

				for relationship in &map.relationships {
					if relationship.from.name == t.name {
						relationships.push(relationship.clone())
					}
				}

				operation_registry.register_entity(t, relationships);
			}
			DbPrimitive::Enum(_) => {}
		}
	}

	let relationships = Arc::new(map.relationships.clone());
	let query_info = SchemaData {
		operation_registry: Arc::new(operation_registry),
		relationships,
		kind: SchemaKind::Query
	};

	let  mutation_info = SchemaData {
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
	Mutation
}

#[derive(Clone)]
pub struct SchemaData<S>
where
	S: ScalarValue + Send + Sync,
{
	kind: SchemaKind,
	operation_registry: Arc<OperationRegistry<S>>,
	relationships: Arc<Vec<DbRelationship>>,
}

pub struct SchemaType;

impl<S> GraphQLType<S> for SchemaType
where
	S: ScalarValue + Send + Sync,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(match info.kind {
			SchemaKind::Query => "Query",
			SchemaKind::Mutation => "Mutation"
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
	S: ScalarValue + Send + Sync,
{
	type Context = ();
	type TypeInfo = SchemaData<S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<S> GraphQLValueAsync<S> for SchemaType
where
	S: ScalarValue + Send + Sync,
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
