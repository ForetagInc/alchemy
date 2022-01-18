use super::Context;

use super::Query;
use super::Mutation;

use juniper::{
	RootNode,
	EmptySubscription
};

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema
{
	Schema::new(
		Query,
		Mutation,
		EmptySubscription::<Context>::new()
	)
}