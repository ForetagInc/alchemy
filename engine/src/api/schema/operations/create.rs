use std::collections::HashMap;

use convert_case::Casing;
use juniper::InputValue;

use crate::api::schema::fields::Entity;
use crate::api::schema::input::insert::{EntityInsert, EntityInsertData, EntityInsertRelationship};
use crate::api::schema::operations::{
	execute_internal_query, execute_query, get_filter_by_indices_attributes, OperationData,
	QueryReturnType,
};
use crate::api::schema::AsyncScalarValue;
use crate::lib::database::aql::{
	AQLFilterOperation, AQLOperation, AQLProperty, AQLQuery, AQLQueryBind, AQLQueryMethod,
	AQLQueryParameter,
};

async fn insert_relationships<S>(
	relationships: Vec<EntityInsertRelationship<S>>,
	key: &str,
	data: &OperationData<S>,
) where
	S: AsyncScalarValue,
{
	for relationship in relationships {
		match relationship {
			EntityInsertRelationship::Existing(k, attributes) => {
				let mut query = AQLQuery::new(0);

				query.filter = Some(get_filter_by_indices_attributes(&attributes));
				query.properties = vec![AQLProperty {
					name: "_id".to_string(),
				}];
				query.limit = Some(1);

				let mut edge = "";
				let mut from_collection = "";
				let mut to_collection = "";

				for relationship in &data.relationships {
					if relationship.name == k {
						edge = &relationship.edge;
						from_collection = &relationship.from.collection_name;
						to_collection = &relationship.to.collection_name;
					}
				}

				let mut insert_query = AQLQuery::new(0);

				insert_query.method = AQLQueryMethod::CreateRelationship(Box::new(query));

				let mut attrs = HashMap::new();

				// Add edge and from attributes
				attrs.insert("@edge".to_string(), InputValue::scalar(edge.to_string()));
				attrs.insert(
					"__from".to_string(),
					InputValue::scalar(format!("{}/{}", from_collection, key)),
				);

				execute_internal_query::<S>(insert_query, to_collection, attributes, attrs).await;
			}
			EntityInsertRelationship::New(_, _) => {}
		}
	}
}

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
				let create_data = execute_internal_query::<S>(insert_query, collection, HashMap::new(), HashMap::new()).await;
				let inserted_key = create_data[0]["_key"].as_str().unwrap().to_string();

				if !object.relationships.is_empty() {
					insert_relationships(object.relationships, &inserted_key, data).await;
				}

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
