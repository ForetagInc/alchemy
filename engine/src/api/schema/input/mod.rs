use juniper::{InputValue, ScalarValue};

pub mod filter;
pub mod insert;
pub mod set;

mod utils;

pub fn to_str<S>(v: &InputValue<S>) -> Option<String>
	where
		S: ScalarValue
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
});

pub fn to_float<S>(v: &InputValue<S>) -> Option<f64>
	where
		S: ScalarValue
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
		S: ScalarValue
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
		S: ScalarValue
{
	v.as_scalar().map(|i| i.as_boolean().unwrap())
}

utils::define_type_filter!(bool, bool, "BoolComparisonExp", to_bool {
	BoolEqual, "_eq", Equal;
});
