use std::fmt::Debug;

use juniper::{InputValue, ScalarValue};

use crate::lib::database::aql::{AQLNode, AQLQueryRaw, AQLQueryValue};

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
	StringLessThan, "_lt", LessThan;
	StringLessOrEqualThan, "_lte", LessOrEqualThan;
	StringNotEqual, "_neq", NotEqual;
	StringNotRegex, "_nregex", NotRegex;
	StringRegex, "_regex", Regex;
	// TODO: In NotIn ILike NotILike SimilarTo NotSimilarTo IRegex NotIRegex

	* StringInArray, "_in", Vec<String>, (attr, val) -> {
		use crate::api::schema::input::{get_list_nodes, to_str};
		use crate::lib::database::aql::{AQLFilterInOperation, AQLQueryParameter};

		let nodes = get_list_nodes(val, to_str);

		Box::new(AQLFilterInOperation {
			left_node: Box::new(AQLQueryParameter(attr.to_string())),
			vec: nodes,
		})
	};
	* StringNotInArray, "_nin", Vec<String>, (attr, val) -> {
		use crate::api::schema::input::{get_list_nodes, to_str};
		use crate::lib::database::aql::{AQLFilterInOperation, AQLQueryParameter, AQLNotFilter};

		let nodes = get_list_nodes(val, to_str);

		Box::new(AQLNotFilter(Box::new(AQLFilterInOperation {
			left_node: Box::new(AQLQueryParameter(attr.to_string())),
			vec: nodes,
		})))
	};
});

pub fn get_list_nodes<S, M, R>(value: &InputValue<S>, mutator: M) -> Vec<Box<dyn AQLNode>>
where
	S: ScalarValue,
	M: Fn(&InputValue<S>) -> Option<R>,
	R: Debug,
{
	let mut nodes: Vec<Box<dyn AQLNode>> = Vec::new();

	if let Some(list) = value.to_list_value() {
		for item in list {
			nodes.push(match mutator(item) {
				None => Box::new(AQLQueryRaw("null".to_string())),
				Some(v) => Box::new(AQLQueryValue(format!("{:?}", v))),
			})
		}
	}

	nodes
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
