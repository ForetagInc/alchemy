use crate::lib::graphql::Query;

use juniper::{
	RootNode,
	EmptyMutation,
	EmptySubscription
};

pub type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

pub fn schema() -> Schema
{
	Schema::new(
		Query,
		EmptyMutation::new(),
		EmptySubscription::new()
	)
}

pub fn generate_schema()
{
	todo!();
}