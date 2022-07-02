pub mod filter;
pub mod insert;
pub mod set;
pub mod string_filter;

macro_rules! define_filter {
	($type:ty, $name:ident, $key:literal, $operation:expr) => {
		pub struct $name;

		impl<S> crate::api::schema::input::filter::FilterOperation<S> for $name
		where
			S: ::juniper::ScalarValue,
		{
			fn get_schema_argument<'r, 'd>(
				registry: &mut ::juniper::executor::Registry<'r, S>,
			) -> ::juniper::meta::Argument<'r, S> {
				registry.arg::<Option<$type>>($key, &())
			}
		}

		impl $name {
			pub fn get_aql_filter_node<S>(
				attribute: &str,
				value: &::juniper::InputValue<S>,
			) -> Box<dyn crate::lib::database::aql::AQLNode>
			where
				S: ::juniper::ScalarValue,
			{
				Box::new(crate::lib::database::aql::AQLFilterOperation {
					left_node: Box::new(crate::lib::database::aql::AQLQueryParameter(
						attribute.to_string(),
					)),
					operation: $operation,
					right_node: match value.as_string_value() {
						None => {
							Box::new(crate::lib::database::aql::AQLQueryRaw("null".to_string()))
						}
						Some(v) => {
							Box::new(crate::lib::database::aql::AQLQueryValue(v.to_string()))
						}
					},
				})
			}
		}
	};
}

pub(crate) use define_filter;
