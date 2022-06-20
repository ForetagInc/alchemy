use std::collections::HashMap;

use convert_case::Casing;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::insert::{EntityInsert, EntityInsertData};
use crate::api::schema::operations::{execute_query, QueryReturnType};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};

crate::api::schema::operations::utils::define_operation!(
	Create {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let mut insert_query = AQLQuery::new(0);

			insert_query.method = AQLQueryMethod::Create;
			insert_query.creates = arguments.get::<EntityInsert>("object").unwrap().attributes;

			execute_query(
				query,
				Some(insert_query),
				entity,
				collection,
				QueryReturnType::Single,
				arguments,
				HashMap::new()
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
		arguments(data, registry) {
			object EntityInsert => &EntityInsertData::new(data, registry)
		},
		return_type -> Entity
	}
);
