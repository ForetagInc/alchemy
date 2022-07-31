use juniper::{InputValue, ScalarValue};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use crate::api::schema::utils::convert_json_to_juniper_value;

pub struct JsonScalar(JsonMap<String, JsonValue>);

impl JsonScalar {
	pub fn new(map: JsonMap<String, JsonValue>) -> Self {
		Self(map)
	}
}

#[juniper::graphql_scalar(description = "GraphQL scalar for JSON dynamic values")]
impl<S> GraphQLScalar for JsonScalar
where
	S: ScalarValue,
{
	fn resolve(&self) -> juniper::Value {
		convert_json_to_juniper_value(&self.0)
	}

	fn from_input_value(value: &InputValue<S>) -> Option<JsonScalar> {
		convert_juniper_obj_to_json(value)
	}

	fn from_str(value: juniper::ScalarToken) -> juniper::ParseScalarResult<S> {
		<String as juniper::ParseScalarValue<S>>::from_str(value)
	}
}

fn convert_juniper_obj_to_json<S>(input: &InputValue<S>) -> Option<JsonScalar>
where
	S: ScalarValue,
{
	fn convert<S>(value: &InputValue<S>) -> JsonValue
	where
		S: ScalarValue,
	{
		match value {
			InputValue::Null => JsonValue::Null,
			InputValue::Scalar(ref s) => {
				if let Some(i) = s.as_int() {
					JsonValue::Number(i.into())
				} else if let Some(f) = s.as_float() {
					JsonValue::Number(JsonNumber::from_f64(f).unwrap_or(0i8.into()))
				} else if let Some(str) = s.as_string() {
					JsonValue::String(str)
				} else if let Some(b) = s.as_boolean() {
					JsonValue::Bool(b)
				} else {
					JsonValue::Null
				}
			}
			InputValue::Enum(v) | InputValue::Variable(v) => JsonValue::String(v.to_string()),
			InputValue::List(l) => {
				let mut arr = Vec::new();

				for i in l {
					arr.push(convert(&i.item))
				}

				JsonValue::Array(arr)
			}
			InputValue::Object(ref o) => {
				let mut map = JsonMap::new();

				for (k, v) in o {
					map.insert(k.item.clone(), convert(&v.item));
				}

				JsonValue::Object(map)
			}
		}
	}

	match input {
		InputValue::Object(ref obj) => {
			let mut map = JsonMap::new();

			for (k, v) in obj {
				map.insert(k.item.clone(), convert(&v.item));
			}

			Some(JsonScalar::new(map))
		}
		_ => None,
	}
}
