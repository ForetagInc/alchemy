use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry, ScalarValue};
use std::marker::PhantomData;

use crate::api::input::string_filter::{StringFilter, StringFilterData};
use crate::api::schema::operations::OperationData;
use crate::lib::database::api::DbScalarType;

pub trait FilterOperation<S>
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S>;
}

pub struct EntityFilterData<'a, S>
where
	S: ScalarValue,
{
	pub name: String,
	pub operation_data: &'a OperationData<S>,
}

pub struct EntityFilter<'a, S: 'a> {
	_marker: PhantomData<&'a S>,
}

impl<'a, S> GraphQLValue<S> for EntityFilter<'a, S>
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = EntityFilterData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> GraphQLType<S> for EntityFilter<'a, S>
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
		let mut args = Vec::new();

		let and = registry.arg::<Option<Vec<Self>>>("_and", info);
		let not = registry.arg::<Option<Self>>("_not", info);
		let or = registry.arg::<Option<Vec<Self>>>("_or", info);

		args.extend([and, not, or]);

		for property in &info.operation_data.entity.properties {
			let arg = match property.scalar_type {
				DbScalarType::String => registry.arg::<StringFilter<'a, S>>(
					property.name.as_str(),
					&StringFilterData::from(info),
				),
				_ => registry.arg::<Option<i32>>(property.name.as_str(), &()),
			};

			args.push(arg)
		}

		registry
			.build_input_object_type::<Self>(info, &args)
			.into_meta()
	}
}

impl<'a, S> FromInputValue<S> for EntityFilter<'a, S>
where
	S: ScalarValue,
{
	fn from_input_value(_: &InputValue<S>) -> Option<Self> {
		Some(Self {
			_marker: Default::default(),
		})
	}
}
