use std::collections::HashMap;

use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{EntityIndicesFilter, EntityIndicesFilterData};
use crate::api::schema::input::set::{EntitySet, EntitySetData};
use crate::api::schema::operations::{
	execute_query, get_filter_by_indices_attributes, QueryReturnType,
};
use crate::lib::database::aql::AQLQueryMethod;

crate::api::schema::operations::utils::define_operation!(
	Update {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let indices_filter = arguments.get::<EntityIndicesFilter<S>>("where").unwrap().indices_arguments;

			query.method = AQLQueryMethod::Update;
			query.filter = Some(get_filter_by_indices_attributes(&indices_filter));
			query.updates = arguments.get::<EntitySet>("_set").unwrap().data;

			Box::pin(async move {
				execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Single,
					HashMap::new()
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
					1,
					false,
				)
			)
		},
		arguments(data, _registry) {
			where EntityIndicesFilter<S> => &EntityIndicesFilterData::<S>::new(data)
			_set EntitySet => &EntitySetData::new(data)
		},
		return_type -> Entity
	}
);
