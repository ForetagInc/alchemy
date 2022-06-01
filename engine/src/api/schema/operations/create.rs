use convert_case::Casing;

use crate::api::schema::fields::{Entity, EntityInsert, EntityInsertData};
use crate::api::schema::operations::{execute_query, QueryReturnType};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};

crate::api::schema::operations::utils::define_operation!(
	Create {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let mut insert_query = AQLQuery::new(0);

			insert_query.method = AQLQueryMethod::Create;
			insert_query.creates = arguments.get::<EntityInsert>("_set").unwrap().data;

			execute_query(
				query,
				Some(insert_query),
				entity,
				collection,
				QueryReturnType::Single,
				arguments,
			)
		},
		name(data) -> {
			format!(
				"create{}",
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
			_set EntityInsert => &EntityInsertData::new(data)
		},
		return_type -> Entity
	}
);
