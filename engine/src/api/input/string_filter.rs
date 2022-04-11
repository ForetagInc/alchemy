use crate::api::input::filter::{EntityFilterData, FilterOperation};
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry, ScalarValue};
use std::marker::PhantomData;

use crate::api::schema::operations::OperationData;
use crate::lib::database::aql::{AQLFilter, AQLLogicalFilter, AQLLogicalOperator, AQLNode, AQLOperation, AQLQueryParameter, AQLQueryRaw, AQLQueryValue};

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

		args.push(StringEq::get_schema_argument(registry));

		registry
			.build_input_object_type::<Self>(info, &args)
			.into_meta()
	}
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

impl<'a, S> StringFilter<'a, S>
where
	S: ScalarValue,
{
	pub fn get_aql_filter_node(attribute: String, value: InputValue<S>) -> impl AQLNode {
		let mut node = AQLLogicalFilter {
			nodes: Vec::new(),
			operation: AQLLogicalOperator::AND,
		};

		match value {
			InputValue::Object(items) => {
				for (key, value) in items {
					match key.item.as_str() {
						"_eq" => node.nodes.push(Box::new(StringEq::get_aql_filter_node(
							&attribute,
							&value.item,
						))),
						_ => {}
					}
				}
			}
			_ => {}
		}

		node
	}
}

pub struct StringEq;

impl<S> FilterOperation<S> for StringEq
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_eq", &())
	}
}

impl StringEq {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> impl AQLNode
	where
		S: ScalarValue,
	{
		let right_node: Box<dyn AQLNode> = match value.as_string_value() {
			None => Box::new(AQLQueryRaw("null".to_string())),
			Some(v) => Box::new(AQLQueryValue(v.to_string()))
		};

		AQLFilter {
			left_node: Box::new(AQLQueryParameter(attribute.to_string())),
			operation: AQLOperation::EQUAL,
			right_node,
		}
	}
}
