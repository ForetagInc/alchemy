use crate::api::input::filter::{EntityFilterData, FilterOperation};
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry, ScalarValue};
use std::marker::PhantomData;

use crate::api::schema::operations::OperationData;

pub struct StringFilterData<'a, S>
where
	S: ScalarValue,
{
	pub operation_data: &'a OperationData<S>,
}

impl<'a, S> StringFilterData<'a, S>
where
	S: ScalarValue,
{
	pub fn from(data: &EntityFilterData<'a, S>) -> Self {
		Self {
			operation_data: data.operation_data,
		}
	}
}

pub struct StringFilter<'a, S: 'a> {
	_marker: PhantomData<&'a S>,
}

impl<'a, S> GraphQLValue<S> for StringFilter<'a, S>
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = StringFilterData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> GraphQLType<S> for StringFilter<'a, S>
where
	S: ScalarValue,
{
	fn name(_: &Self::TypeInfo) -> Option<&str> {
		Some("StringComparisonExp")
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut args = Vec::new();

		args.push(create_filter_arg::<StringEq, S>(registry));

		registry
			.build_input_object_type::<Self>(info, &args)
			.into_meta()
	}
}

fn create_filter_arg<'r, T, S>(registry: &mut Registry<'r, S>) -> Argument<'r, S>
where
	S: 'r + ScalarValue,
	T: FilterOperation<S>,
{
	T::get_schema_argument(registry)
}

impl<'a, S> FromInputValue<S> for StringFilter<'a, S>
where
	S: ScalarValue,
{
	fn from_input_value(_: &InputValue<S>) -> Option<Self> {
		Some(Self {
			_marker: Default::default(),
		})
	}
}

pub struct StringEq;

impl<S> FilterOperation<S> for StringEq
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<String>("_eq", &())
	}
}
