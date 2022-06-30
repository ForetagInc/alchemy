use std::collections::HashMap;

use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use crate::api::schema::input::set::{EntitySet, EntitySetData};
use crate::api::schema::operations::{
	execute_internal_query, execute_query, get_filter_in_keys, QueryReturnType,
};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};

crate::api::schema::operations::utils::define_operation!(
	UpdateAll {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let mut update_query = AQLQuery::new(0);

			update_query.method = AQLQueryMethod::Update(arguments.get::<EntitySet>("_set").unwrap().data);
			update_query.filter = get_aql_filter_from_args(arguments, data);
			update_query.limit = arguments.get::<i32>("limit");

			Box::pin(async move {
				let create_data = execute_internal_query::<S>(update_query, collection, HashMap::new(), HashMap::new()).await;
				let mut keys = Vec::new();

				for row in create_data {
					keys.push(row["_key"].as_str().unwrap().to_string());
				}

				query.filter = Some(get_filter_in_keys());

				let mut args = HashMap::new();

				args.insert("_keys".to_string(), keys);

				execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Multiple,
					HashMap::new(),
					args
				).await
			})
		},
		name(data) -> {
			format!(
				"update{}",
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
			limit Option<i32> => &()
			where EntityFilter<S> => &EntityFilterData::new(data)
			_set EntitySet => &EntitySetData::new(data)
		},
		return_type -> Vec<Entity>
	}
);
