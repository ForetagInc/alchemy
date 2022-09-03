use std::collections::HashMap;

use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{EntityIndicesFilter, EntityIndicesFilterData};
use crate::api::schema::operations::{
	execute_query, get_filter_by_indices_attributes, QueryReturnType,
};

crate::api::schema::operations::utils::define_operation!(
	Get {
		on_call(data, args, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let indices_filter = args.get::<EntityIndicesFilter<S>>("where").unwrap().indices_arguments;

			query.filter = Some(get_filter_by_indices_attributes(&indices_filter));
			query.limit = Some(1);

			Box::pin(async move {
				execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Single,
					indices_filter,
					HashMap::<String, String>::new()
				).await
			})
		},
		name(data) -> {
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
		},
		arguments(data, _registry) {
			where EntityIndicesFilter<S> => &EntityIndicesFilterData::new(data)
		},
		return_type -> Option<Entity>
	}
);
