use crate::api::schema::input::define_filter;
use crate::api::schema::input::filter::{EntityFilterData, FilterOperation};
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry, ScalarValue};
use std::marker::PhantomData;

use crate::api::schema::operations::OperationData;
use crate::lib::database::aql::{
	AQLFilterOperation, AQLLogicalFilter, AQLLogicalOperator, AQLNode, AQLOperation,
	AQLQueryParameter, AQLQueryRaw, AQLQueryValue,
};

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

		args.push(StringEqual::get_schema_argument(registry));
		args.push(StringGreaterThan::get_schema_argument(registry));
		args.push(StringGreaterOrEqualThan::get_schema_argument(registry));
		args.push(StringLike::get_schema_argument(registry));
		args.push(StringLessThan::get_schema_argument(registry));
		args.push(StringLessOrEqualThan::get_schema_argument(registry));
		args.push(StringNotEqual::get_schema_argument(registry));
		args.push(StringNotLike::get_schema_argument(registry));
		args.push(StringNotRegex::get_schema_argument(registry));
		args.push(StringRegex::get_schema_argument(registry));

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
	pub fn get_aql_filter_node(attribute: String, value: &InputValue<S>) -> impl AQLNode {
		let mut node = AQLLogicalFilter {
			nodes: Vec::new(),
			operation: AQLLogicalOperator::AND,
		};

		match value {
			InputValue::Object(items) => {
				for (key, value) in items {
					node.nodes.push(match key.item.as_str() {
						"_eq" => StringEqual::get_aql_filter_node(&attribute, &value.item),
						"_gt" => StringGreaterThan::get_aql_filter_node(&attribute, &value.item),
						"_gte" => {
							StringGreaterOrEqualThan::get_aql_filter_node(&attribute, &value.item)
						}
						"_like" => StringLike::get_aql_filter_node(&attribute, &value.item),
						"_lt" => StringLessThan::get_aql_filter_node(&attribute, &value.item),
						"_lte" => {
							StringLessOrEqualThan::get_aql_filter_node(&attribute, &value.item)
						}
						"_neq" => StringNotEqual::get_aql_filter_node(&attribute, &value.item),
						"_nlike" => StringNotLike::get_aql_filter_node(&attribute, &value.item),
						"_nregex" => StringNotRegex::get_aql_filter_node(&attribute, &value.item),
						"_regex" => StringRegex::get_aql_filter_node(&attribute, &value.item),
						_ => unreachable!(),
					});
				}
			}
			_ => {}
		}

		node
	}
}

fn get_aql_value<S>(value: &InputValue<S>) -> Box<dyn AQLNode>
where
	S: ScalarValue,
{
	match value.as_string_value() {
		None => Box::new(AQLQueryRaw("null".to_string())),
		Some(v) => Box::new(AQLQueryValue(v.to_string())),
	}
}

fn get_aql_parameter(param: &str) -> Box<AQLQueryParameter> {
	Box::new(AQLQueryParameter(param.to_string()))
}

fn get_filter_operation<S>(
	param: &str,
	operation: AQLOperation,
	value: &InputValue<S>,
) -> Box<dyn AQLNode>
where
	S: ScalarValue,
{
	Box::new(AQLFilterOperation {
		left_node: get_aql_parameter(param),
		operation,
		right_node: get_aql_value(value),
	})
}

define_filter!(String, StringEqual, "_eq", AQLOperation::Equal);
define_filter!(String, StringGreaterThan, "_gt", AQLOperation::GreaterThan);
define_filter!(
	String,
	StringGreaterOrEqualThan,
	"_gte",
	AQLOperation::GreaterOrEqualThan
);
define_filter!(String, StringLike, "_like", AQLOperation::Like);
define_filter!(String, StringLessThan, "_lt", AQLOperation::LessThan);
define_filter!(
	String,
	StringLessOrEqualThan,
	"_lte",
	AQLOperation::LessOrEqualThan
);
define_filter!(String, StringNotEqual, "_neq", AQLOperation::NotEqual);
define_filter!(String, StringNotLike, "_nlike", AQLOperation::NotLike);
define_filter!(String, StringNotRegex, "_nregex", AQLOperation::NotRegex);
define_filter!(String, StringRegex, "_regex", AQLOperation::Regex);
