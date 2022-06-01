use convert_case::Casing;
use juniper::ID;

use crate::api::schema::fields::{Entity, EntitySet, EntitySetData};
use crate::api::schema::operations::{execute_query, get_by_key_filter, QueryReturnType};
use crate::lib::database::aql::AQLQueryMethod;

crate::api::schema::operations::utils::define_operation!(
	Update {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			query.method = AQLQueryMethod::Update;
			query.filter = Some(get_by_key_filter());
			query.updates = arguments.get::<EntitySet>("_set").unwrap().data;

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
		arguments(data) {
			_key ID => &()
			_set EntitySet => &EntitySetData::new(data)
		},
		return_type -> Entity
	}
);
