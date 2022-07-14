macro_rules! define_filter {
	($type:ty, $name:ident, $key:literal, $operation:ident, $fn:ident) => {
		pub struct $name;

		impl<S> crate::api::schema::input::filter::FilterOperation<S> for $name
		where
			S: ::juniper::ScalarValue,
		{
			fn get_schema_argument<'r, 'd>(
				registry: &mut ::juniper::Registry<'r, S>,
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
					operation: crate::lib::database::aql::AQLOperation::$operation,
					right_node: match crate::api::schema::input::$fn(value) {
						None => {
							Box::new(crate::lib::database::aql::AQLQueryRaw("null".to_string()))
						}
						Some(v) => {
							Box::new(crate::lib::database::aql::AQLQueryValue(format!("{:?}", v)))
						}
					},
				})
			}
		}
	};
}

pub(crate) use define_filter;

macro_rules! define_type_filter {
	($name:ident, $type:ty, $type_name:literal, $fn:ident {
		$(
			$filter_name:ident, $key:literal, $operation:ident;
		)*
	}) => {
		mod $name {
			use crate::api::schema::input::filter::FilterOperation;

			pub struct FilterData<'a, S>
			where
				S: ::juniper::ScalarValue,
			{
				pub operation_data: &'a crate::api::schema::operations::OperationData<S>,
			}

			impl<'a, S> FilterData<'a, S>
			where
				S: ::juniper::ScalarValue,
			{
				pub fn from(data: &crate::api::schema::input::filter::EntityFilterData<'a, S>) -> Self {
					Self {
						operation_data: data.operation_data,
					}
				}
			}

			pub struct Filter<'a, S: 'a> {
				_marker: ::std::marker::PhantomData<&'a S>,
			}

			impl<'a, S> ::juniper::GraphQLValue<S> for Filter<'a, S>
			where
				S: ::juniper::ScalarValue,
			{
				type Context = ();
				type TypeInfo = FilterData<'a, S>;

				fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
					<Self as ::juniper::GraphQLType<S>>::name(info)
				}
			}

			impl<'a, S> ::juniper::GraphQLType<S> for Filter<'a, S>
			where
				S: ::juniper::ScalarValue,
			{
				fn name(_: &Self::TypeInfo) -> Option<&str> {
					Some($type_name)
				}

				fn meta<'r>(info: &Self::TypeInfo, registry: &mut ::juniper::Registry<'r, S>) -> ::juniper::meta::MetaType<'r, S>
				where
					S: 'r,
				{
					let mut args = Vec::new();

					$(
						args.push($filter_name::get_schema_argument(registry));
					)*

					registry
						.build_input_object_type::<Self>(info, &args)
						.into_meta()
				}
			}

			impl<'a, S> ::juniper::FromInputValue<S> for Filter<'a, S>
			where
				S: ::juniper::ScalarValue,
			{
				fn from_input_value(_: &::juniper::InputValue<S>) -> Option<Self> {
					Some(Self {
						_marker: Default::default(),
					})
				}
			}

			impl<'a, S> Filter<'a, S>
			where
				S: ::juniper::ScalarValue,
			{
				pub fn get_aql_filter_node(attribute: String, value: &::juniper::InputValue<S>) -> impl crate::lib::database::aql::AQLNode {
					let mut node = crate::lib::database::aql::AQLLogicalFilter {
						nodes: Vec::new(),
						operation: crate::lib::database::aql::AQLLogicalOperator::AND,
					};

					match value {
						::juniper::InputValue::Object(items) => {
							for (key, value) in items {
								node.nodes.push(match key.item.as_str() {
									$(
										$key => $filter_name::get_aql_filter_node(&attribute, &value.item),
									)*
									_ => unreachable!(),
								});
							}
						}
						_ => {}
					}

					node
				}
			}

			$(
				crate::api::schema::input::utils::define_filter!(
					$type,
					$filter_name,
					$key,
					$operation,
					$fn
				);
			)*
		}
	};
}
pub(crate) use define_type_filter;
