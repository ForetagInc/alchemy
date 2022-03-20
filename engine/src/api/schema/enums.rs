use juniper::{FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry, ScalarValue};
use juniper::meta::{EnumValue, MetaType};

pub struct DbEnumInfo {
	pub(crate) name: String,
	pub(crate) properties: Vec<String>
}

pub struct GraphQLEnum(String);

impl<S> GraphQLValue<S> for GraphQLEnum
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = DbEnumInfo;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<S> GraphQLType<S> for GraphQLEnum
where
	S: ScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str>
	where
		Self: Sized,
	{
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
		Self: Sized,
	{
		registry
			.build_enum_type::<GraphQLEnum>(
				&info,
				&info
					.properties
					.iter()
					.map(|p| EnumValue::new(p))
					.collect::<Vec<EnumValue>>()
					.as_slice(),
			)
			.into_meta()
	}
}

impl<S> FromInputValue<S> for GraphQLEnum
where
	S: ScalarValue,
{
	fn from_input_value(v: &InputValue<S>) -> Option<Self> {
		v.as_enum_value()
			.or_else(|| v.as_string_value())
			.map(|v| Self(v.to_string()))
	}
}
