use super::Query;
use super::Mutation;

use juniper::{
	RootNode,
	EmptySubscription
};

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription>;

pub fn schema() -> Schema
{
	Schema::new(
		Query,
		Mutation,
		EmptySubscription::new()
	)
}

pub fn generate_schema()
{
	todo!();
}