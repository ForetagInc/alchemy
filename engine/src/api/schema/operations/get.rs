use convert_case::Casing;
use juniper::ID;

use crate::api::schema::fields::Entity;
use crate::api::schema::operations::{
	execute_query, get_by_id_filter, QueryReturnType,
};

use crate::api::schema::operations::utils::*;

define_operation!(
	Get {
		on_call(data, args, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			query.filter = Some(get_by_id_filter());
			query.limit = Some(1);

			execute_query(
				query,
				entity,
				collection,
				QueryReturnType::Single,
				args,
			)
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
		arguments(_data) {
			ID id &()
		},
		return_type -> Option<Entity>
	}
);
