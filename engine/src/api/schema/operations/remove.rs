use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{EntityIndicesFilter, EntityIndicesFilterData};
use crate::api::schema::operations::{
	execute_query, get_filter_by_indices_attributes, QueryReturnType,
};
use crate::lib::database::aql::AQLQueryMethod;

crate::api::schema::operations::utils::define_operation!(
	Remove {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let indices_filter = arguments.get::<EntityIndicesFilter<S>>("where").unwrap().indices_arguments;

			query.method = AQLQueryMethod::Remove;
			query.filter = Some(get_filter_by_indices_attributes(&indices_filter));

			execute_query(
				query,
				None,
				entity,
				collection,
				QueryReturnType::Single,
				arguments,
				indices_filter
			)
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
			where EntityIndicesFilter<S> => &EntityIndicesFilterData::<S>::new(data)
		},
		return_type -> Entity
	}
);
