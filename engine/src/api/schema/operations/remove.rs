use convert_case::Casing;
use juniper::ID;

use crate::api::schema::fields::Entity;
use crate::api::schema::operations::{execute_query, get_by_key_filter, QueryReturnType};
use crate::lib::database::aql::AQLQueryMethod;

crate::api::schema::operations::utils::define_operation!(
	Remove {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			query.method = AQLQueryMethod::Remove;
			query.filter = Some(get_by_key_filter());

			execute_query(
				query,
				None,
				entity,
				collection,
				QueryReturnType::Single,
				arguments,
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
		arguments(_data) {
			_key ID => &()
		},
		return_type -> Entity
	}
);
