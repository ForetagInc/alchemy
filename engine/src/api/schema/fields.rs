use crate::api::schema::enums::{DbEnumInfo, GraphQLEnum};
use juniper::meta::{Field, MetaType};
use juniper::{Arguments, BoxFuture, ExecutionResult, Executor, GraphQLType, GraphQLValue, GraphQLValueAsync, Object, Registry, ScalarValue, Selection, Spanning, Value};
use std::collections::HashMap;

use crate::api::schema::operations::{Operation, OperationData};
use crate::lib::database::api::{
	DbEntity, DbProperty, DbRelationship, DbRelationshipType, DbScalarType,
};

pub struct QueryFieldFactory;

impl QueryFieldFactory {
	pub fn new<'a, S, T>(
		name: &str,
		operation: &Box<T>,
		registry: &mut Registry<'a, S>,
	) -> Field<'a, S>
	where
		S: ScalarValue,
		T: Operation<S> + ?Sized,
	{
		let mut field = registry.field::<QueryField<S>>(name, &operation.get_data());

		for arg in operation.get_arguments(registry) {
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

pub struct QueryField<S> {
	pub properties: HashMap<String, Box<dyn GraphQLValue<S, Context = (), TypeInfo = ()> + Send>>,
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
	info: &OperationData,
) -> Field<'r, S>
where
	S: ScalarValue,
{
	return match relationship.relationship_type {
		DbRelationshipType::OneToOne => {
			registry.field::<QueryField<S>>(relationship.name.as_str(), info)
		}
		DbRelationshipType::OneToMany | DbRelationshipType::ManyToMany => {
			registry.field::<Vec<QueryField<S>>>(relationship.name.as_str(), info)
		}
	};
}

impl<S> GraphQLType<S> for QueryField<S>
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
			.build_object_type::<QueryField<S>>(info, &fields)
			.into_meta()
	}
}

impl<S> GraphQLValue<S> for QueryField<S>
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = OperationData;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}

	fn resolve_field(
		&self,
		_info: &Self::TypeInfo,
		field_name: &str,
		_arguments: &Arguments<S>,
		executor: &Executor<Self::Context, S>,
	) -> ExecutionResult<S> {
		let value = self.properties.get(field_name).unwrap();

		executor.resolve(&(), &value)
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
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = ();

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
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
		executor: &'b Executor<Self::Context, S>,
	) -> BoxFuture<'b, ExecutionResult<S>> {
		println!(
			"{}\n{:#?}\n{:#?}",
			self.field_name, selection_set, self.arguments
		);

		Box::pin(resolve_graphql_field(
			self.field_name,
			self.arguments,
			selection_set.unwrap(),
			executor
		))
	}
}

#[async_recursion::async_recursion]
async fn resolve_graphql_field<'a, S>(
	field_name: &str,
	arguments: &'a Arguments<'a, S>,
	selection_set: &'a [Selection<'a, S>],
	executor: &'a Executor<'a, 'a, (), S>,
) -> ExecutionResult<S>
where
	S: ScalarValue + Send + Sync,
{
	use juniper::futures::stream::{FuturesOrdered, StreamExt as _};

	let mut object = Object::<S>::with_capacity(selection_set.len());

	for selection in selection_set {
		match *selection {
			Selection::Field(Spanning {
								 item: ref f,
								 ..
							 }) => {

				let response_name = f.alias.as_ref().unwrap_or(&f.name).item;

				if f.name.item == "__typename" {
					continue;
				}

				let response_name = response_name.to_string();

				println!("{}", response_name);

				if let Some(inner_selection_set) = &f.selection_set {
					resolve_graphql_field(field_name, arguments, inner_selection_set, executor).await?;
				}
			}
			_ => unreachable!()
		}
	}

	Ok(juniper::Value::<S>::Object(juniper::Object::with_capacity(10)))
}
