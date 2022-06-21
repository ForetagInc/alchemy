use std::collections::HashMap;

use convert_case::Casing;
use juniper::InputValue;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::insert::{EntityInsert, EntityInsertData};
use crate::api::schema::operations::{execute_internal_query, execute_query, QueryReturnType};
use crate::lib::database::aql::{
	AQLFilterOperation, AQLOperation, AQLQuery, AQLQueryBind, AQLQueryMethod, AQLQueryParameter,
};

crate::api::schema::operations::utils::define_operation!(
	Create {
		on_call(data, arguments, query) -> {
			let entity = &data.entity;
			let collection = &entity.collection_name;

			let mut insert_query = AQLQuery::new(0);
			let object = arguments.get::<EntityInsert<S>>("object").unwrap();

			insert_query.method = AQLQueryMethod::Create;
			insert_query.creates = object.attributes;

			Box::pin(async move {
				let create_data = execute_internal_query::<S>(insert_query, collection, HashMap::new()).await;
				let inserted_key = create_data[0]["_key"].as_str().unwrap().to_string();

				query.filter = Some(Box::new(AQLFilterOperation {
					left_node: Box::new(AQLQueryParameter("_key".to_string())),
					operation: AQLOperation::Equal,
					right_node: Box::new(AQLQueryBind("_key".to_string())),
				}));

				let mut args = HashMap::new();

				args.insert("_key".to_string(), InputValue::scalar(inserted_key));

				execute_query(
					query,
					entity,
					collection,
					QueryReturnType::Single,
					args
				).await
			})
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
			object EntityInsert<S> => &EntityInsertData::new(data, registry)
		},
		return_type -> Entity
	}
);
