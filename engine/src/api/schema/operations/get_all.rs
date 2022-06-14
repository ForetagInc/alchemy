use std::collections::HashMap;

use convert_case::Casing;

use crate::api::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use crate::api::schema::fields::Entity;
use crate::api::schema::operations::{execute_query, QueryReturnType};

crate::api::schema::operations::utils::define_operation!(
	GetAll {
		on_call(data, args, query) -> {
			let time = std::time::Instant::now();

			let entity = &data.entity;
			let collection = &entity.collection_name;

			query.limit = args.get::<i32>("limit");
			query.filter = get_aql_filter_from_args(args, data);

			println!("Query AQL Filter generation: {:?}", time.elapsed());

			execute_query(
				query,
				None,
				entity,
				collection,
				QueryReturnType::Multiple,
				args,
				HashMap::new()
			)
		},
		name(data) -> {
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
		},
		arguments(data, _registry) {
			limit Option<i32> => &()
			where Option<EntityFilter<S>> => &EntityFilterData::new(data)
		},
		return_type -> Vec<Entity>
	}
);
