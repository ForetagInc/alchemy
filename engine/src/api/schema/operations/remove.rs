use std::collections::HashMap;

use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{EntityIndicesFilter, EntityIndicesFilterData};
use crate::api::schema::operations::{
	execute_internal_query, execute_query, get_filter_by_indices_attributes, QueryReturnType,
};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};

crate::api::schema::operations::utils::define_operation!(
	Remove {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let indices_filter = arguments.get::<EntityIndicesFilter<S>>("where").unwrap().indices_arguments;

			query.filter = Some(get_filter_by_indices_attributes(&indices_filter));

			Box::pin(async move {
				let result = execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Single,
					indices_filter.clone(),
					HashMap::<String, String>::new()
				).await;

				let mut remove_query = AQLQuery::new(0);

				remove_query.method = AQLQueryMethod::Remove;
				remove_query.filter = Some(get_filter_by_indices_attributes(&indices_filter));

				execute_internal_query::<S>(remove_query, collection, indices_filter, HashMap::new()).await;

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
					1,
					false,
				)
			)
		},
		arguments(data, _registry) {
			where EntityIndicesFilter<S> => &EntityIndicesFilterData::new(data)
		},
		return_type -> Entity
	}
);
