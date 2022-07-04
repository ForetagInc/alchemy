pub mod filter;
pub mod insert;
pub mod set;

mod utils;

utils::define_type_filter!(str, String, "StringComparisonExp", as_string_value {
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

utils::define_type_filter!(float, f64, "FloatComparisonExp", as_float_value {
	FloatEqual, "_eq", Equal;
	FloatGreaterThan, "_gt", GreaterThan;
	FloatGreaterOrEqualThan, "_gte", GreaterOrEqualThan;
	FloatLessThan, "_lt", LessThan;
	FloatLessOrEqualThan, "_lte", LessOrEqualThan;
	FloatNotEqual, "_neq", NotEqual;
	// TODO: In NotIn
});

utils::define_type_filter!(int, i32, "IntComparisonExp", as_int_value {
	IntEqual, "_eq", Equal;
	IntGreaterThan, "_gt", GreaterThan;
	IntGreaterOrEqualThan, "_gte", GreaterOrEqualThan;
	IntLessThan, "_lt", LessThan;
	IntLessOrEqualThan, "_lte", LessOrEqualThan;
	IntNotEqual, "_neq", NotEqual;
	// TODO: In NotIn
});
