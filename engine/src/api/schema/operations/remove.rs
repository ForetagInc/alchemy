use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, ID, Registry, ScalarValue};
use rust_arango::{AqlQuery, ClientError};
use serde_json::Value as JsonValue;

use crate::api::schema::fields::{Entity, EntityData};
use crate::api::schema::operations::{FutureType, Operation, OperationData, OperationRegistry, get_single_entry, get_by_id_filter};
use crate::lib::database::aql::{AQLQuery, AQLQueryMethod};
use crate::lib::database::DATABASE;

pub struct Remove;

impl<S> Operation<S> for Remove
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

		query.method = AQLQueryMethod::Remove;
		query.filter = Some(get_by_id_filter());

		println!("Query AQL Filter generation: {:?}", time.elapsed());

		Box::pin(async move {
			let query_str = query.to_aql();

			println!("{}", &query_str);

			let entries_query = AqlQuery::builder()
				.query(&query_str)
				.bind_var("@collection".to_string(), collection.clone())
				.bind_var(
					query.get_argument_key("id"),
					arguments.get::<String>("id").unwrap(),
				);

			let entries: Result<Vec<JsonValue>, ClientError> = DATABASE
				.get()
				.await
				.database
				.aql_query(entries_query.build())
				.await;

			println!("SQL: {:?}", time.elapsed());

			get_single_entry(entries, entity.name.clone())
		})
	}

	fn get_operation_name(data: &OperationData<S>) -> String {
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
	}

	fn get_arguments<'r, 'd>(
		registry: &mut Registry<'r, S>,
		_: &'d OperationData<S>,
	) -> Vec<Argument<'r, S>> {
		vec![registry.arg::<ID>("id", &())]
	}

	fn build_field<'r>(
		registry: &mut Registry<'r, S>,
		name: &str,
		data: &OperationData<S>,
		operation_registry: &OperationRegistry<S>,
	) -> Field<'r, S> {
		registry.field::<Entity>(
			name,
			&EntityData {
				data,
				registry: operation_registry,
			},
		)
	}
}
