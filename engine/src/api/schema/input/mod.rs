use juniper::meta::Argument;
use juniper::{InputValue, Registry, ScalarValue};

use crate::api::schema::input::filter::FilterOperation;
use crate::lib::database::aql::{
	AQLFilterInOperation, AQLFilterOperation, AQLNode, AQLOperation, AQLQueryParameter,
	AQLQueryRaw, AQLQueryValue,
};

pub mod filter;
pub mod insert;
pub mod set;

mod utils;

pub fn to_str<S>(v: &InputValue<S>) -> Option<String>
where
	S: ScalarValue,
{
	v.as_string_value().map(|i| i.to_string())
}

utils::define_type_filter!(str, String, "StringComparisonExp", to_str {
	StringEqual, "_eq", Equal;
	StringGreaterThan, "_gt", GreaterThan;
	StringGreaterOrEqualThan, "_gte", GreaterOrEqualThan;
	StringLike, "_like", Like;
	StringLessThan, "_lt", LessThan;
	StringLessOrEqualThan, "_lte", LessOrEqualThan;
	StringNotEqual, "_neq", NotEqual;
	StringNotLike, "_nlike", NotLike;
	StringNotRegex, "_nregex", NotRegex;
	StringRegex, "_regex", Regex;
	// TODO: In NotIn ILike NotILike SimilarTo NotSimilarTo IRegex NotIRegex

	* StringInArray, "_in";
	* StringNotInArray, "_nin";
});

fn get_list_nodes<S>(value: &InputValue<S>) -> Vec<Box<dyn AQLNode>>
where
	S: ScalarValue,
{
	let mut nodes: Vec<Box<dyn AQLNode>> = Vec::new();

	if let Some(list) = value.to_list_value() {
		for item in list {
			nodes.push(match to_str(item) {
				None => Box::new(AQLQueryRaw("null".to_string())),
				Some(v) => Box::new(AQLQueryValue(format!("{:?}", v))),
			})
		}
	}

	nodes
}

pub struct StringInArray;

impl<S> FilterOperation<S> for StringInArray
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<Vec<String>>>("_in", &())
	}
}

impl StringInArray {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		let nodes = get_list_nodes(value);

		Box::new(AQLFilterInOperation {
			left_node: Box::new(AQLQueryParameter(attribute.to_string())),
			vec: nodes,
			not: false,
		})
	}
}

pub struct StringNotInArray;

impl<S> FilterOperation<S> for StringNotInArray
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S> {
		registry.arg::<Option<Vec<String>>>("_nin", &())
	}
}

impl StringNotInArray {
	pub fn get_aql_filter_node<S>(attribute: &str, value: &InputValue<S>) -> Box<dyn AQLNode>
	where
		S: ScalarValue,
	{
		let nodes = get_list_nodes(value);

		Box::new(AQLFilterInOperation {
			left_node: Box::new(AQLQueryParameter(attribute.to_string())),
			vec: nodes,
			not: true,
		})
	}
}

pub fn to_float<S>(v: &InputValue<S>) -> Option<f64>
where
	S: ScalarValue,
{
	v.as_float_value()
}

utils::define_type_filter!(float, f64, "FloatComparisonExp", to_float {
	FloatEqual, "_eq", Equal;
	FloatGreaterThan, "_gt", GreaterThan;
	FloatGreaterOrEqualThan, "_gte", GreaterOrEqualThan;
	FloatLessThan, "_lt", LessThan;
	FloatLessOrEqualThan, "_lte", LessOrEqualThan;
	FloatNotEqual, "_neq", NotEqual;
	// TODO: In NotIn
});

pub fn to_int<S>(v: &InputValue<S>) -> Option<i32>
where
	S: ScalarValue,
{
	v.as_int_value()
}

utils::define_type_filter!(int, i32, "IntComparisonExp", to_int {
	IntEqual, "_eq", Equal;
	IntGreaterThan, "_gt", GreaterThan;
	IntGreaterOrEqualThan, "_gte", GreaterOrEqualThan;
	IntLessThan, "_lt", LessThan;
	IntLessOrEqualThan, "_lte", LessOrEqualThan;
	IntNotEqual, "_neq", NotEqual;
	// TODO: In NotIn
});

pub fn to_bool<S>(v: &InputValue<S>) -> Option<bool>
where
	S: ScalarValue,
{
	v.as_scalar().map(|i| i.as_boolean().unwrap())
}

utils::define_type_filter!(bool, bool, "BoolComparisonExp", to_bool {
	BoolEqual, "_eq", Equal;
});
