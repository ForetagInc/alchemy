use std::collections::HashMap;

use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use crate::api::schema::operations::{execute_query, QueryReturnType};
use crate::lib::database::aql::AQLQueryMethod;

crate::api::schema::operations::utils::define_operation!(
	RemoveAll {
		on_call(data, arguments, query) -> {
			let time = std::time::Instant::now();

			let entity = &data.entity;
			let collection = &entity.collection_name;

			query.method = AQLQueryMethod::Remove;
			query.filter = get_aql_filter_from_args(arguments, data);
			query.limit = arguments.get::<i32>("limit");

			println!("Query AQL Filter generation: {:?}", time.elapsed());

			Box::pin(async move {
				execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Multiple,
					HashMap::new()
				).await
			})
		},
		name(data) -> {
			format!(
				"remove{}",
				pluralizer::pluralize(
					data.entity
						.name
						.to_case(convert_case::Case::Pascal)
						.as_str(),
					2,
					false,
				)
			)
		},
		arguments(data, _registry) {
			where EntityFilter<S> => &EntityFilterData::new(data)
			limit Option<i32> => &()
		},
		return_type -> Vec<Entity>
	}
);
