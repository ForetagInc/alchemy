use crate::api::schema::enums::{DbEnumInfo, GraphQLEnum};
use juniper::meta::{Field, MetaType};
use juniper::{
	Arguments, ExecutionResult, Executor, GraphQLType, GraphQLValue, Registry, ScalarValue,
};
use std::collections::HashMap;

use crate::api::schema::operations::Operation;
use crate::lib::database::api::{DbEntity, DbProperty, DbScalarType};

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
		let mut field = registry.field::<QueryField<S>>(name, &operation.get_entity());

		for arg in operation.get_arguments(registry) {
			field = field.argument(arg);
		}

		field
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

impl<S> GraphQLType<S> for QueryField<S>
where
	S: ScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut fields = Vec::new();

		for property in &info.properties {
			let field = build_field_from_property(registry, &property, &property.scalar_type, true);

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
	type TypeInfo = DbEntity;

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
