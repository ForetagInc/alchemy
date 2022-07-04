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
});
