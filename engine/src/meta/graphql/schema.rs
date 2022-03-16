use super::Context;

use super::Mutation;
use super::Query;

use juniper::{EmptySubscription, RootNode};

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
	Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
