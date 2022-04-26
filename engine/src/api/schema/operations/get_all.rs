use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, Registry, ScalarValue};

use crate::api::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use crate::api::schema::fields::{Entity, EntityData};
use crate::api::schema::operations::{
	execute_query, FutureType, Operation, OperationData, OperationRegistry, QueryReturnType,
};
use crate::lib::database::aql::AQLQuery;

pub struct GetAll;

impl<S> Operation<S> for GetAll
where
	S: ScalarValue + Send + Sync,
{
	fn call<'b>(
		data: &'b OperationData<S>,
		arguments: &'b Arguments<S>,
		mut query: AQLQuery,
	) -> FutureType<'b, S> {
		let time = std::time::Instant::now();

		let entity = &data.entity;
		let collection = &entity.collection_name;

		query.limit = arguments.get::<i32>("limit");
		query.filter = get_aql_filter_from_args(arguments, data);

		println!("Query AQL Filter generation: {:?}", time.elapsed());

		execute_query(
			query,
			entity,
			collection,
			QueryReturnType::Multiple,
			arguments,
		)
	}

	fn get_operation_name(data: &OperationData<S>) -> String {
		format!(
			"getAll{}",
			pluralizer::pluralize(
				data.entity
					.name
					.to_case(convert_case::Case::Pascal)
					.as_str(),
				2,
				false,
			)
		)
	}

	fn get_arguments<'r, 'd>(
		registry: &mut Registry<'r, S>,
		data: &'d OperationData<S>,
	) -> Vec<Argument<'r, S>> {
		vec![
			registry.arg::<Option<i32>>("limit", &()),
			registry.arg::<Option<EntityFilter<'d, S>>>("where", &EntityFilterData::new(data)),
		]
	}

	fn build_field<'r>(
		registry: &mut Registry<'r, S>,
		name: &str,
		data: &OperationData<S>,
		operation_registry: &OperationRegistry<S>,
	) -> Field<'r, S> {
		registry.field::<Vec<Entity>>(
			name,
			&EntityData {
				data,
				registry: operation_registry,
			},
		)
	}
}
