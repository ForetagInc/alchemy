macro_rules! define_operation {
	(
		$name:ident {
			on_call($data:ident , $arguments:ident , $query:ident) -> $call_body:block,
			name($name_data:ident) -> $name_body:block,
			arguments($args_data:ident, $args_registry:ident) {
				$(
					$arg_name:ident $arg_type:ty => $arg_info:expr
				)*
			},
			return_type -> $ret_type:ty
		}
	) => {
		pub struct $name;

		impl<S> crate::api::schema::operations::Operation<S> for $name
		where
			S: crate::api::schema::AsyncScalarValue,
		{
			fn call<'b>(
				data: &'b crate::api::schema::operations::OperationData<S>,
				arguments: &'b ::juniper::Arguments<S>,
				query: crate::lib::database::aql::AQLQuery,
			) -> crate::api::schema::operations::FutureType<'b, S> {
				let $data = data;
				let $arguments = arguments;
				let mut $query = query;

				$call_body
			}

			fn get_operation_name(data: &crate::api::schema::operations::OperationData<S>) -> String {
				let $name_data = data;

				$name_body
			}

			fn get_arguments<'r, 'd>(
				registry: &mut ::juniper::Registry<'r, S>,
				data: &'d crate::api::schema::operations::OperationData<S>,
				operation_registry: &crate::api::schema::operations::OperationRegistry<S>,
			) -> Vec<::juniper::meta::Argument<'r, S>> {
				let $args_data = data;
				let $args_registry = operation_registry;

				vec![
					$(
						registry.arg::<$arg_type>(stringify!($arg_name), $arg_info),
					)*
				]
			}

			fn build_field<'r>(
				registry: &mut ::juniper::Registry<'r, S>,
				name: &str,
				data: &crate::api::schema::operations::OperationData<S>,
				operation_registry: &crate::api::schema::operations::OperationRegistry<S>,
			) -> ::juniper::meta::Field<'r, S> {
				registry.field::<$ret_type>(
					name,
					&crate::api::schema::fields::EntityData {
						data,
						registry: operation_registry,
					},
				)
			}
		}
	}
}

macro_rules! assign_parameters {
	($args:expr, ($k:ident, $v:ident) -> $closure:tt) => {
		for (key, value) in $args {
			let $k = key.clone();

			match value {
				::juniper::InputValue::Scalar(s) => {
					if let Some(int) = s.as_int() {
						let $v = int;

						$closure
					} else if let Some(float) = s.as_float() {
						let $v = float;

						$closure
					} else if let Some(str) = s.as_string() {
						let $v = str;

						$closure
					}
				}
				_ => {
					println!(
						"WARN: Using non-scalar for query arguments ({}, {})",
						key,
						value.to_string()
					)
				}
			}
		}
	};
}

pub(crate) use assign_parameters;
pub(crate) use define_operation;
