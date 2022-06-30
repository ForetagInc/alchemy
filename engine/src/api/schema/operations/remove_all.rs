use std::collections::HashMap;

use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use crate::api::schema::operations::{execute_internal_query, execute_query, QueryReturnType};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};

crate::api::schema::operations::utils::define_operation!(
	RemoveAll {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			query.filter = get_aql_filter_from_args(arguments, data);
			query.limit = arguments.get::<i32>("limit");

			Box::pin(async move {
				let result = execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Multiple,
					HashMap::new(),
					HashMap::<String, String>::new()
				).await;

				let mut remove_query = AQLQuery::new(0);

				remove_query.method = AQLQueryMethod::Remove;
				remove_query.filter = get_aql_filter_from_args(arguments, data);
				remove_query.limit = arguments.get::<i32>("limit");

				execute_internal_query::<S>(remove_query, collection, HashMap::new(), HashMap::new()).await;

				result
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
