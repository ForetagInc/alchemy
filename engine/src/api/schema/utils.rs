use juniper::{Object, ScalarValue, Value};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

fn convert_number<S>(n: &JsonNumber) -> Value<S>
where
	S: ScalarValue,
{
	return if n.is_i64() {
		let v = n.as_i64().unwrap();

		let res = if v > i32::MAX as i64 {
			i32::MAX
		} else if v < i32::MIN as i64 {
			i32::MIN
		} else {
			v as i32
		};

		Value::scalar(res)
	} else if n.is_u64() {
		let v = n.as_u64().unwrap();

		let res = if v > i32::MAX as u64 {
			i32::MAX
		} else if v < i32::MIN as u64 {
			i32::MIN
		} else {
			v as i32
		};

		Value::scalar(res)
	} else {
		let v = n.as_f64().unwrap();

		Value::scalar(v)
	};
}

pub fn convert_json_to_juniper_value<S>(data: &JsonMap<String, JsonValue>) -> Value<S>
where
	S: ScalarValue,
{
	let mut object = Object::<S>::with_capacity(data.len());

	fn convert<S>(val: &JsonValue) -> Value<S>
	where
		S: ScalarValue,
	{
		match val {
			JsonValue::Null => Value::null(),
			JsonValue::Bool(v) => Value::scalar(v.to_owned()),
			JsonValue::Number(n) => convert_number(n),
			JsonValue::String(s) => Value::scalar(s.to_owned()),
			JsonValue::Array(a) => Value::list(a.iter().map(|i| convert(i)).collect()),
			JsonValue::Object(ref o) => convert_json_to_juniper_value(o),
		}
	}

	for (key, val) in data {
		object.add_field(key, convert(val));
	}

	Value::Object(object)
}
