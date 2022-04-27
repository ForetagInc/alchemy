macro_rules! define_operation {
	(
		$name:ident {
			on_call($data:ident , $arguments:ident , $query:ident) -> $call_body:block,
			name($name_data:ident) -> $name_body:block,
			arguments($args_data:ident) {
				$(
					$arg_type:ident$(<$arg_generics:ident>)? $arg_name:ident $arg_info:expr
				)*
			},
			return_type -> $ret_type:ident$(<$ret_generics:ident>)?
		}
	) => {
		pub struct $name;

		impl<S> crate::api::schema::operations::Operation<S> for $name
		where
			S: ::juniper::ScalarValue + Send + Sync,
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
			) -> Vec<::juniper::meta::Argument<'r, S>> {
				let $args_data = data;

				vec![
					$(
						registry.arg::<$arg_type $(<$arg_generics>)?>(stringify!($arg_name), $arg_info),
					)*
				]
			}

			fn build_field<'r>(
				registry: &mut ::juniper::Registry<'r, S>,
				name: &str,
				data: &crate::api::schema::operations::OperationData<S>,
				operation_registry: &crate::api::schema::operations::OperationRegistry<S>,
			) -> ::juniper::meta::Field<'r, S> {
				registry.field::<$ret_type $(<$ret_generics>)?>(
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

pub(crate) use define_operation;