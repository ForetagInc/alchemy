use juniper::meta::{Field, MetaType};
use juniper::{
	Arguments, BoxFuture, ExecutionResult, Executor, GraphQLType, GraphQLValue, GraphQLValueAsync,
	Registry, ScalarValue, Selection, Spanning, Value,
};
use std::marker::PhantomData;

use crate::api::schema::enums::{DbEnumInfo, GraphQLEnum};
use crate::api::schema::input::filter::{get_aql_filter_from_args, EntityFilter, EntityFilterData};
use crate::api::schema::operations::{OperationData, OperationEntry, OperationRegistry};
use crate::api::schema::{AsyncScalarValue, SchemaData};
use crate::lib::database::api::{DbProperty, DbRelationship, DbScalarType};
use crate::lib::database::aql::{AQLProperty, AQLQuery, AQLQueryRelationship};

pub struct SchemaFieldFactory;

impl SchemaFieldFactory {
	pub fn new<'a, S>(
		name: &str,
		operation: &OperationEntry<S>,
		registry: &mut Registry<'a, S>,
		operation_registry: &OperationRegistry<S>,
	) -> Field<'a, S>
	where
		S: AsyncScalarValue,
	{
		let mut field =
			(operation.field_closure)(registry, name, &operation.data, operation_registry);

		for arg in (operation.arguments_closure)(registry, &operation.data, operation_registry) {
			field = field.argument(arg);
		}

		field
	}

	pub fn new_resolver<'a, S>(
		field_name: &'a str,
		arguments: &'a Arguments<S>,
	) -> SchemaFieldResolver<'a, S>
	where
		S: AsyncScalarValue,
	{
		SchemaFieldResolver {
			field_name,
			arguments,
		}
	}
}

pub struct EntityData<'a, S>
where
	S: AsyncScalarValue,
{
	pub registry: &'a OperationRegistry<S>,
	pub data: &'a OperationData,
}

pub struct Entity<'a> {
	_marker: PhantomData<&'a ()>,
}

fn build_field_from_property<'r, S>(
	registry: &mut Registry<'r, S>,
	property: &DbProperty,
	scalar_type: &DbScalarType,
	enforce_required: bool,
) -> Field<'r, S>
where
	S: ScalarValue,
{
	let required = property.required;

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

			if required && enforce_required {
				field.field_type = juniper::Type::NonNullList(Box::new(field.field_type));
			} else {
				field.field_type = juniper::Type::List(Box::new(field.field_type));
			}

			field
		}

		DbScalarType::Enum(values) => build_field::<GraphQLEnum, S>(
			registry,
			property,
			required,
			&DbEnumInfo {
				name: property.associated_type.clone().unwrap(),
				properties: values.clone(),
			},
		),
		DbScalarType::String => build_field::<String, S>(registry, property, required, &()),
		DbScalarType::Object => build_field::<String, S>(registry, property, required, &()),
		DbScalarType::Float => build_field::<f64, S>(registry, property, required, &()),
		DbScalarType::Int => build_field::<i32, S>(registry, property, required, &()),
		DbScalarType::Boolean => build_field::<bool, S>(registry, property, required, &()),
	}
}

fn build_field_from_relationship<'r, S>(
	registry: &mut Registry<'r, S>,
	relationship: &DbRelationship,
	info: &EntityData<S>,
) -> Field<'r, S>
where
	S: AsyncScalarValue,
{
	let returns_array = relationship.relationship_type.returns_array();

	let field = if returns_array {
		registry.field::<Vec<Entity>>(relationship.name.as_str(), info)
	} else {
		registry.field::<Entity>(relationship.name.as_str(), info)
	};

	if returns_array {
		field
			.argument(
				registry.arg::<Option<EntityFilter<S>>>("where", &EntityFilterData::new(info.data)),
			)
			.argument(registry.arg::<Option<i32>>("limit", &()))
	} else {
		field
	}
}

impl<'a, S> GraphQLType<S> for Entity<'a>
where
	S: AsyncScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.data.entity.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut fields = Vec::new();

		for property in &info.data.entity.properties {
			let field = build_field_from_property(registry, &property, &property.scalar_type, true);

			fields.push(field);
		}

		for relationship in &*info.data.relationships {
			let rel_info = &EntityData {
				data: &*info
					.registry
					.get_operation_data(&relationship.to.name)
					.expect("Relationship entity operation data not found"),
				registry: info.registry,
			};

			let field = build_field_from_relationship(registry, relationship, rel_info);

			fields.push(field);
		}

		registry
			.build_object_type::<Entity>(info, &fields)
			.into_meta()
	}
}

impl<'a, S> GraphQLValue<S> for Entity<'a>
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = EntityData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

/// Phantom GraphQLValue just to implement field resolution
/// This type won't be shown on the Schema
pub struct SchemaFieldResolver<'a, S>
where
	S: ScalarValue,
{
	field_name: &'a str,
	arguments: &'a Arguments<'a, S>,
}

impl<'a, S> GraphQLValue<S> for SchemaFieldResolver<'a, S>
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = SchemaData<S>;

	fn type_name<'i>(&self, _: &'i Self::TypeInfo) -> Option<&'i str> {
		None
	}
}

impl<'a, S> GraphQLValueAsync<S> for SchemaFieldResolver<'a, S>
where
	S: AsyncScalarValue,
{
	fn resolve_async<'b>(
		&'b self,
		info: &'b Self::TypeInfo,
		selection_set: Option<&'b [Selection<S>]>,
		executor: &'b Executor<Self::Context, S>,
	) -> BoxFuture<'b, ExecutionResult<S>> {
		Box::pin(resolve_graphql_field(
			info,
			self.field_name,
			self.arguments,
			selection_set.unwrap(),
			executor,
		))
	}
}

async fn resolve_graphql_field<'a, S>(
	info: &'a SchemaData<S>,
	field_name: &str,
	arguments: &'a Arguments<'a, S>,
	selection_set: &'a [Selection<'a, S>],
	executor: &'a Executor<'a, 'a, <SchemaFieldResolver<'a, S> as GraphQLValue<S>>::Context, S>,
) -> ExecutionResult<S>
where
	S: AsyncScalarValue,
{
	if let Some(entry) = info.operation_registry.get_operation(field_name) {
		let query =
			get_query_from_graphql(selection_set, &entry.data.entity.name, info, None, executor);

		(entry.closure)(&entry.data, arguments, query).await
	} else {
		Ok(Value::null())
	}
}

fn get_query_from_graphql<'a, S>(
	selection_set: &'a [Selection<'a, S>],
	entity_name: &'a str,
	data: &'a SchemaData<S>,
	query_id: Option<u32>,
	executor: &'a Executor<'a, 'a, <SchemaFieldResolver<'a, S> as GraphQLValue<S>>::Context, S>,
) -> AQLQuery
where
	S: AsyncScalarValue,
{
	let mut query = AQLQuery::new(query_id.unwrap_or(1));

	let meta_type = executor
		.schema()
		.concrete_type_by_name(entity_name.as_ref())
		.expect("Type not found in schema");

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
						executor,
					);

					let meta_field = meta_type.field_by_name(f.name.item).unwrap_or_else(|| {
						panic!(
							"Field {} not found on type {:?}",
							f.name.item,
							meta_type.name()
						)
					});

					let args = Arguments::new(
						f.arguments.as_ref().map(|m| {
							m.item
								.iter()
								.map(|&(ref k, ref v)| {
									(k.item, v.item.clone().into_const(executor.variables()))
								})
								.collect()
						}),
						&meta_field.arguments,
					);

					let operation_data = data
						.operation_registry
						.get_operation_data(meta_field.field_type.innermost_name())
						.unwrap();

					inner_query.limit = args.get::<i32>("limit");
					inner_query.filter = get_aql_filter_from_args(&args, &operation_data);

					for relationship in &*data.relationships {
						if relationship.name == response_name {
							inner_query.relationship = Some(AQLQueryRelationship {
								edge: relationship.edge.clone(),
								variable_name: query.get_variable_name(),
								direction: relationship.direction.clone(),
								relationship_type: relationship.relationship_type.clone(),
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
