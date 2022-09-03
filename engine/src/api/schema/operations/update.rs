use std::collections::HashMap;

use convert_case::Casing;
use juniper::InputValue;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::filter::{EntityIndicesFilter, EntityIndicesFilterData};
use crate::api::schema::input::set::{EntitySet, EntitySetData};
use crate::api::schema::operations::{
	execute_internal_query, execute_query, get_filter_by_indices_attributes, get_filter_by_key,
	QueryReturnType,
};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};

crate::api::schema::operations::utils::define_operation!(
	Update {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let indices_filter = arguments.get::<EntityIndicesFilter<S>>("where").unwrap().indices_arguments;

			let mut update_query = AQLQuery::new(0);

			update_query.method = AQLQueryMethod::Update(arguments.get::<EntitySet>("_set").unwrap().data);
			update_query.filter = Some(get_filter_by_indices_attributes(&indices_filter));

			Box::pin(async move {
				let create_data = execute_internal_query::<S>(update_query, collection, indices_filter, HashMap::new()).await;
				let inserted_key = create_data[0]["_key"].as_str().unwrap().to_string();

				query.filter = Some(get_filter_by_key());

				let mut args = HashMap::new();

				args.insert("_key".to_string(), InputValue::scalar(inserted_key));

				execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Single,
					args,
					HashMap::<String, String>::new()
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
			where EntityIndicesFilter<S> => &EntityIndicesFilterData::new(data)
			_set EntitySet => &EntitySetData::new(data)
		},
		return_type -> Entity
	}
);
