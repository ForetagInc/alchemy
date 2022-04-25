use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, Registry, ScalarValue};
use rust_arango::{AqlQuery, ClientError};
use serde_json::Value as JsonValue;
use crate::api::input::filter::{EntityFilter, EntityFilterData, get_aql_filter_from_args};

use crate::api::schema::fields::{Entity, EntityData, EntitySet, EntitySetData};
use crate::api::schema::operations::{FutureType, Operation, OperationData, OperationRegistry, get_multiple_entries};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};
use crate::lib::database::DATABASE;

pub struct UpdateAll;

impl<S> Operation<S> for UpdateAll
where
	S: ScalarValue + Send + Sync,
{
	fn call<'b>(
		data: &'b OperationData<S>,
		arguments: &'b Arguments<S>,
		mut query: AQLQuery,
	) -> FutureType<'b, S> {
		let time = std::time::Instant::now();

		let entity = &data.entity;
		let collection = &entity.collection_name;

		query.method = AQLQueryMethod::Update;
		query.filter = get_aql_filter_from_args(arguments, data);
		query.limit = arguments.get::<i32>("limit");
		query.updates = arguments.get::<EntitySet>("_set").unwrap().data;

		println!("Query AQL Filter generation: {:?}", time.elapsed());

		Box::pin(async move {
			let query_str = query.to_aql();

			println!("{}", &query_str);

			let entries_query = AqlQuery::builder()
				.query(&query_str)
				.bind_var("@collection".to_string(), collection.clone());

			let entries: Result<Vec<JsonValue>, ClientError> = DATABASE
				.get()
				.await
				.database
				.aql_query(entries_query.build())
				.await;

			println!("SQL: {:?}", time.elapsed());

			get_multiple_entries(entries)
		})
	}

	fn get_operation_name(data: &OperationData<S>) -> String {
		format!(
			"updateAll{}",
			pluralizer::pluralize(
				data.entity
					.name
					.to_case(convert_case::Case::Pascal)
					.as_str(),
				2,
				false,
			)
		)
	}

	fn get_arguments<'r, 'd>(
		registry: &mut Registry<'r, S>,
		data: &'d OperationData<S>,
	) -> Vec<Argument<'r, S>> {
		vec![
			registry.arg::<EntitySet>(
				"_set",
				&EntitySetData {
					name: format!("{}Set", data.entity.name.as_str()),
					data,
				},
			),
			registry.arg::<Option<i32>>("limit", &()),
			registry.arg::<EntityFilter<'d, S>>("where", &EntityFilterData::new(data)),
		]
	}

	fn build_field<'r>(
		registry: &mut Registry<'r, S>,
		name: &str,
		data: &OperationData<S>,
		operation_registry: &OperationRegistry<S>,
	) -> Field<'r, S> {
		registry.field::<Vec<Entity>>(
			name,
			&EntityData {
				data,
				registry: operation_registry,
			},
		)
	}
}
