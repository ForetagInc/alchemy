use crate::api::schema::enums::{DbEnumInfo, GraphQLEnum};
use juniper::meta::{Field, MetaType};
use juniper::{
	Arguments, BoxFuture, ExecutionResult, Executor, GraphQLType, GraphQLValue, GraphQLValueAsync,
	Registry, ScalarValue, Selection, Spanning, Value,
};

use crate::api::schema::operations::{OperationData, OperationEntry};
use crate::api::schema::{owns_relationship, QueryData};
use crate::lib::database::api::{DbProperty, DbRelationship, DbRelationshipType, DbScalarType};
use crate::lib::database::aql::{AQLProperty, AQLQuery, AQLQueryRelationship};

pub struct QueryFieldFactory;

impl QueryFieldFactory {
	pub fn new<'a, S>(
		name: &str,
		operation: &OperationEntry<S>,
		registry: &mut Registry<'a, S>,
	) -> Field<'a, S>
	where
		S: ScalarValue,
	{
		let field_builder = operation.field_closure;

		let mut field = field_builder(registry, name, &operation.data);

		let args = operation.arguments_closure;

		for arg in args(registry) {
			field = field.argument(arg);
		}

		field
	}

	pub fn new_resolver<'a, S>(
		field_name: &'a str,
		arguments: &'a Arguments<S>,
	) -> QueryFieldResolver<'a, S>
	where
		S: ScalarValue + Send + Sync,
	{
		QueryFieldResolver {
			field_name,
			arguments,
		}
	}
}

pub struct Entity;

fn build_field_from_property<'r, S>(
	registry: &mut Registry<'r, S>,
	property: &DbProperty,
	scalar_type: &DbScalarType,
	enforce_required: bool,
) -> Field<'r, S>
where
	S: ScalarValue,
{
	fn build_field<'r, T, S>(
		registry: &mut Registry<'r, S>,
		property: &DbProperty,
		required: bool,
		info: &T::TypeInfo,
	) -> Field<'r, S>
	where
		S: ScalarValue + 'r,
		T: GraphQLType<S, Context = ()>,
	{
		let is_array = matches!(property.scalar_type, DbScalarType::Array(_));

		if required && !is_array {
			registry.field::<T>(property.name.as_str(), info)
		} else {
			registry.field::<Option<T>>(property.name.as_str(), info)
		}
	}

	match scalar_type {
		DbScalarType::Array(t) => {
			let mut field = build_field_from_property(registry, property, &t, false);

			if property.required && enforce_required {
				field.field_type = juniper::Type::NonNullList(Box::new(field.field_type));
			} else {
				field.field_type = juniper::Type::List(Box::new(field.field_type));
			}

			field
		}

		DbScalarType::Enum(values) => build_field::<GraphQLEnum, S>(
			registry,
			property,
			property.required,
			&DbEnumInfo {
				name: property.associated_type.clone().unwrap(),
				properties: values.clone(),
			},
		),
		DbScalarType::String => {
			build_field::<String, S>(registry, property, property.required, &())
		}
		DbScalarType::Object => {
			build_field::<String, S>(registry, property, property.required, &())
		}
		DbScalarType::Float => build_field::<f64, S>(registry, property, property.required, &()),
		DbScalarType::Int => build_field::<i32, S>(registry, property, property.required, &()),
		DbScalarType::Boolean => build_field::<bool, S>(registry, property, property.required, &()),
	}
}

fn build_field_from_relationship<'r, S>(
	registry: &mut Registry<'r, S>,
	relationship: &DbRelationship,
	info: &OperationData<S>,
) -> Field<'r, S>
where
	S: ScalarValue,
{
	return match relationship.relationship_type {
		DbRelationshipType::OneToOne => registry.field::<Entity>(relationship.name.as_str(), info),
		DbRelationshipType::OneToMany | DbRelationshipType::ManyToMany => {
			registry.field::<Vec<Entity>>(relationship.name.as_str(), info)
		}
	};
}

impl<S> GraphQLType<S> for Entity
where
	S: ScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.entity.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut fields = Vec::new();

		for property in &info.entity.properties {
			let field = build_field_from_property(registry, &property, &property.scalar_type, true);

			fields.push(field);
		}

		for relationship in &*info.relationships {
			let field = build_field_from_relationship(registry, relationship, &info);

			fields.push(field);
		}

		registry
			.build_object_type::<Entity>(info, &fields)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for Entity
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = OperationData<S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

/// Phantom GraphQLValue just to implement field resolution
/// This type won't be shown on the Schema
pub struct QueryFieldResolver<'a, S>
where
	S: ScalarValue,
{
	field_name: &'a str,
	arguments: &'a Arguments<'a, S>,
}

impl<'a, S> GraphQLValue<S> for QueryFieldResolver<'a, S>
where
	S: ScalarValue + Send + Sync,
{
	type Context = ();
	type TypeInfo = QueryData<S>;

	fn type_name<'i>(&self, _: &'i Self::TypeInfo) -> Option<&'i str> {
		None
	}
}

impl<'a, S> GraphQLValueAsync<S> for QueryFieldResolver<'a, S>
where
	S: ScalarValue + Send + Sync,
{
	fn resolve_async<'b>(
		&'b self,
		info: &'b Self::TypeInfo,
		selection_set: Option<&'b [Selection<S>]>,
		_executor: &'b Executor<Self::Context, S>,
	) -> BoxFuture<'b, ExecutionResult<S>> {
		Box::pin(resolve_graphql_field(
			info,
			self.field_name,
			self.arguments,
			selection_set.unwrap(),
		))
	}
}

async fn resolve_graphql_field<'a, S>(
	info: &'a QueryData<S>,
	field_name: &str,
	arguments: &'a Arguments<'a, S>,
	selection_set: &'a [Selection<'a, S>],
) -> ExecutionResult<S>
where
	S: ScalarValue + Send + Sync,
{
	if let Some(entry) = info.operation_registry.get_operation(field_name) {
		let query = get_query_from_graphql(selection_set, &entry.data.entity.name, info, None);

		let closure = entry.closure;

		closure(&entry.data, arguments, query).await
	} else {
		Ok(Value::null())
	}
}

fn get_query_from_graphql<'a, S>(
	selection_set: &'a [Selection<'a, S>],
	entity_name: &'a str,
	data: &'a QueryData<S>,
	query_id: Option<u32>,
) -> AQLQuery<'a>
where
	S: ScalarValue + Send + Sync,
{
	let mut query = AQLQuery::new(query_id.unwrap_or(1));

	for selection in selection_set {
		match *selection {
			Selection::Field(Spanning { item: ref f, .. }) => {
				let response_name = f.alias.as_ref().unwrap_or(&f.name).item;

				if f.name.item == "__typename" {
					continue;
				}

				let response_name = response_name.to_string();

				if let Some(inner_selection_set) = &f.selection_set {
					let mut inner_query = get_query_from_graphql(
						inner_selection_set,
						entity_name,
						data,
						Some(query.id + 1),
					);

					for relationship in &data.relationships {
						if owns_relationship(&relationship, entity_name) {
							inner_query.relationship = Some(AQLQueryRelationship {
								edge: relationship.edge.clone(),
								variable_name: query.get_variable_name(),
								direction: relationship.direction.clone(),
							});

							query.relations.insert(response_name.clone(), inner_query);

							break;
						}
					}
				} else {
					query.properties.push(AQLProperty {
						name: response_name,
					});
				}
			}
			_ => unreachable!(),
		}
	}

	query
}
