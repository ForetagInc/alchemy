use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, Registry, ScalarValue, ID};

use crate::api::schema::fields::{Entity, EntityData};
use crate::api::schema::operations::{
	execute_query, get_by_id_filter, FutureType, Operation, OperationData, OperationRegistry,
	QueryReturnType,
};
use crate::lib::database::aql::AQLQuery;

pub struct Get;

impl<S> Operation<S> for Get
where
	S: ScalarValue + Send + Sync,
{
	fn call<'b>(
		data: &'b OperationData<S>,
		arguments: &'b Arguments<S>,
		mut query: AQLQuery,
	) -> FutureType<'b, S> {
		let entity = &data.entity;
		let collection = &entity.collection_name;

		query.filter = Some(get_by_id_filter());
		query.limit = Some(1);

		execute_query(
			query,
			entity,
			collection,
			QueryReturnType::Single,
			arguments,
		)
	}

	fn get_operation_name(data: &OperationData<S>) -> String {
		format!(
			"get{}",
			pluralizer::pluralize(
				data.entity
					.name
					.to_case(convert_case::Case::Pascal)
					.as_str(),
				1,
				false,
			)
		)
	}

	fn get_arguments<'r, 'd>(
		registry: &mut Registry<'r, S>,
		_: &'d OperationData<S>,
	) -> Vec<Argument<'r, S>> {
		vec![registry.arg::<ID>("id", &())]
	}

	fn build_field<'r>(
		registry: &mut Registry<'r, S>,
		name: &str,
		data: &OperationData<S>,
		operation_registry: &OperationRegistry<S>,
	) -> Field<'r, S> {
		registry.field::<Option<Entity>>(
			name,
			&EntityData {
				data,
				registry: operation_registry,
			},
		)
	}
}
