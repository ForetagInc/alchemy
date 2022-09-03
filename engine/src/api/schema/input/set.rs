use std::marker::PhantomData;

use juniper::meta::MetaType;
use juniper::{FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry};

use crate::api::schema::operations::OperationData;
use crate::api::schema::{build_argument_from_property, input_value_to_string, AsyncScalarValue};

pub struct EntitySet<'a> {
	pub data: String,

	_marker: PhantomData<&'a ()>,
}

pub struct EntitySetData<'a> {
	pub name: String,
	pub data: &'a OperationData,
}

impl<'a> EntitySetData<'a> {
	pub fn new(data: &'a OperationData) -> Self {
		Self {
			name: format!("{}Set", data.entity.name.as_str()),
			data,
		}
	}
}

impl<'a, S> GraphQLType<S> for EntitySet<'a>
where
	S: AsyncScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut args = Vec::new();

		for property in &info.data.entity.properties {
			if property.name.eq("_key") {
				continue;
			}

			let arg =
				build_argument_from_property(registry, &property, &property.scalar_type, false);

			args.push(arg);
		}

		registry
			.build_input_object_type::<EntitySet>(info, &args)
			.into_meta()
	}
}

impl<'a, S> GraphQLValue<S> for EntitySet<'a>
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = EntitySetData<'a>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> FromInputValue<S> for EntitySet<'a>
where
	S: AsyncScalarValue,
{
	fn from_input_value(data: &InputValue<S>) -> Option<Self> {
		Some(Self {
			data: input_value_to_string(data),

			_marker: Default::default(),
		})
	}
}
