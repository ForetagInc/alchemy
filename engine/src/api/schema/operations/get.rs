use convert_case::Casing;
use juniper::meta::{Argument, Field};
use juniper::{Arguments, IntoFieldError, Registry, ScalarValue, ID};
use rust_arango::{AqlQuery, ClientError};
use serde_json::Value as JsonValue;

use crate::api::schema::errors::NotFoundError;
use crate::api::schema::fields::{Entity, EntityData};
use crate::api::schema::operations::{
	convert_json_to_juniper_value, FutureType, Operation, OperationData, OperationRegistry,
};
use crate::lib::database::aql::{
	AQLFilterOperation, AQLOperation, AQLQuery, AQLQueryBind, AQLQueryParameter,
};
use crate::lib::database::DATABASE;

pub struct Get;

impl<S> Operation<S> for Get
where
	S: ScalarValue + Send + Sync,
{
	fn call<'b>(
		data: &'b OperationData<S>,
		arguments: &'b Arguments<S>,
		mut query: AQLQuery<'b>,
	) -> FutureType<'b, S> {
		let time = std::time::Instant::now();

		let entity = &data.entity;
		let collection = &entity.collection_name;

		query.filter = Some(Box::new(AQLFilterOperation {
			left_node: Box::new(AQLQueryParameter("_key".to_string())),
			operation: AQLOperation::Equal,
			right_node: Box::new(AQLQueryBind("id".to_string())),
		}));
		query.limit = Some(1);

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

			let not_found_error = NotFoundError::new(entity.name.clone()).into_field_error();

			println!("SQL: {:?}", time.elapsed());

			return match entries {
				Ok(data) => {
					if let Some(first) = data.first() {
						let time2 = std::time::Instant::now();

						let ret = Ok(convert_json_to_juniper_value(first.as_object().unwrap()));

						println!("Conversion: {:?}", time2.elapsed());

						return ret;
					}

					Err(not_found_error)
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
		registry.field::<Option<Entity>>(
			name,
			&EntityData {
				data,
				registry: operation_registry,
			},
		)
	}
}
