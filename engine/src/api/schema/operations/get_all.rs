use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, IntoFieldError, Registry, ScalarValue, Value};
use rust_arango::{AqlQuery, ClientError};
use serde_json::Value as JsonValue;

use crate::api::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use crate::api::schema::errors::NotFoundError;
use crate::api::schema::fields::{Entity, EntityData};
use crate::api::schema::operations::{
	convert_json_to_juniper_value, FutureType, Operation, OperationData, OperationRegistry,
};
use crate::lib::database::aql::AQLQuery;
use crate::lib::database::DATABASE;

pub struct GetAll;

impl<S> Operation<S> for GetAll
where
	S: ScalarValue + Send + Sync,
{
	fn call<'b>(
		data: &'b OperationData<S>,
		arguments: &'b Arguments<S>,
		mut query: AQLQuery<'b>,
	) -> FutureType<'b, S> {
		let mut time = std::time::Instant::now();

		let entity = &data.entity;
		let collection = &entity.collection_name;

		query.limit = arguments.get::<i32>("limit");
		query.filter = get_aql_filter_from_args(arguments, data);

		println!("Query AQL Filter generation: {:?}", time.elapsed());

		Box::pin(async move {
			time = std::time::Instant::now();

			let query_str = query.to_aql();

			println!("Query AQL string generation: {:?}", time.elapsed());

			println!("{}", &query_str);

			let entries_query = AqlQuery::builder()
				.query(&query_str)
				.bind_var("@collection".to_string(), collection.clone());

			time = std::time::Instant::now();

			let entries: Result<Vec<JsonValue>, ClientError> = DATABASE
				.get()
				.await
				.database
				.aql_query(entries_query.build())
				.await;

			let not_found_error = NotFoundError::new(entity.name.clone()).into_field_error();

			println!("SQL: {:?}", time.elapsed());

			return match entries {
				Ok(data) => {
					let mut output = Vec::<Value<S>>::new();

					time = std::time::Instant::now();

					for datum in data {
						output.push(convert_json_to_juniper_value(datum.as_object().unwrap()));
					}

					println!("Output conversion: {:?}", time.elapsed());

					Ok(Value::list(output))
				}
				Err(e) => {
					println!("{:?}", e);

					Err(not_found_error)
				}
			};
		})
	}

	fn get_operation_name(data: &OperationData<S>) -> String {
		format!(
			"getAll{}",
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
			registry.arg::<Option<i32>>("limit", &()),
			registry.arg::<Option<EntityFilter<'d, S>>>("where", &EntityFilterData::new(data)),
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
