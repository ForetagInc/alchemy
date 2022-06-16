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
		args.push(StringLike::get_schema_argument(registry));
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

pub struct StringEqual;

impl<S> FilterOperation<S> for StringEqual
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_eq", &())
	}
}

impl StringEqual {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::Equal, value)
	}
}

pub struct StringGreaterThan;

impl<S> FilterOperation<S> for StringGreaterThan
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_gt", &())
	}
}

impl StringGreaterThan {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::GreaterThan, value)
	}
}

pub struct StringGreaterOrEqualThan;

impl<S> FilterOperation<S> for StringGreaterOrEqualThan
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_gte", &())
	}
}

impl StringGreaterOrEqualThan {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::GreaterOrEqualThan, value)
	}
}

pub struct StringLike;

impl<S> FilterOperation<S> for StringLike
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_like", &())
	}
}

impl StringLike {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::Like, value)
	}
}

pub struct StringLessThan;

impl<S> FilterOperation<S> for StringLessThan
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_lt", &())
	}
}

impl StringLessThan {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::LessThan, value)
	}
}

pub struct StringLessOrEqualThan;

impl<S> FilterOperation<S> for StringLessOrEqualThan
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_lte", &())
	}
}

impl StringLessOrEqualThan {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::LessOrEqualThan, value)
	}
}

pub struct StringNotEqual;

impl<S> FilterOperation<S> for StringNotEqual
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_neq", &())
	}
}

impl StringNotEqual {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::NotEqual, value)
	}
}

pub struct StringNotLike;

impl<S> FilterOperation<S> for StringNotLike
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_nlike", &())
	}
}

impl StringNotLike {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::NotLike, value)
	}
}

pub struct StringNotRegex;

impl<S> FilterOperation<S> for StringNotRegex
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_nregex", &())
	}
}

impl StringNotRegex {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::NotRegex, value)
	}
}

pub struct StringRegex;

impl<S> FilterOperation<S> for StringRegex
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<String>>("_regex", &())
	}
}

impl StringRegex {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		get_filter_operation(attribute, AQLOperation::Regex, value)
	}
}
