use crate::api::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, Registry, ScalarValue};

use crate::api::schema::fields::{Entity, EntityData};
use crate::api::schema::operations::{
	execute_query, FutureType, Operation, OperationData, OperationRegistry, QueryReturnType,
};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};

pub struct RemoveAll;

impl<S> Operation<S> for RemoveAll
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

		query.method = AQLQueryMethod::Remove;
		query.filter = get_aql_filter_from_args(arguments, data);
		query.limit = arguments.get::<i32>("limit");

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
			"removeAll{}",
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
			registry.arg::<EntityFilter<'d, S>>("where", &EntityFilterData::new(data)),
			registry.arg::<Option<i32>>("limit", &()),
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
