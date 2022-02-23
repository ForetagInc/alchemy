use serde_json::Value;

use crate::lib::schema::get_all_entries;

const ERR_CHILD_NOT_DEFINED: &str = "ERROR: Child type not defined";

struct GraphQLProperty {
	name: String,
	scalar_type: ScalarType
}

struct ScalarType {
	raw: RawScalarType,
	child: Option<RawScalarType>
}

#[derive(PartialEq)]
pub enum RawScalarType {
	Array,
	Nullable,
	String,
	Object,
	Float,
	Int,
	Boolean,
	Enum,
}

impl From<JsonType> for ScalarType {
	fn from(json_type: JsonType) -> Self {
		match json_type.raw {
			RawJsonType::Array | RawJsonType::Enum => Self {
				raw: json_type.raw.into(),
				child: Some(json_type.child.expect(ERR_CHILD_NOT_DEFINED).into())
			},
			_ => Self {
				raw: json_type.raw.into(),
				child: None
			}
		}
	}
}

impl From<RawJsonType> for RawScalarType {
	fn from(raw_type: RawJsonType) -> Self {
		match raw_type {
			RawJsonType::Array => RawScalarType::Array,
			RawJsonType::Enum => RawScalarType::Enum,
			RawJsonType::Boolean => RawScalarType::Boolean,
			RawJsonType::Integer => RawScalarType::Int,
			RawJsonType::Number => RawScalarType::Float,
			RawJsonType::Object => RawScalarType::Object,
			RawJsonType::String => RawScalarType::String,
			RawJsonType::Nullable => RawScalarType::Nullable
		}
	}
}

struct JsonType {
	raw: RawJsonType,
	child: Option<RawJsonType>
}

#[derive(Copy, Clone)]
pub enum RawJsonType {
	Array,
	Enum,
	Boolean,
	Nullable,
	Integer,
	Number,
	Object,
	String,
}

impl From<&str> for RawJsonType {
	fn from(json_type: &str) -> Self {
		match json_type {
			"array" => Self::Array,
			"enum" => Self::Enum,
			"boolean" => Self::Boolean,
			"integer" => Self::Integer,
			"number" => Self::Number,
			"object" => Self::Object,
			"string" => Self::String,
			_ => Self::Nullable
		}
	}
}

pub async fn generate_sdl()
{
	let entries = get_all_entries().await;

	let mut sdl = String::new();

	println!("----- SDL GENERATION -----");

	for entry in entries.clone().iter()
	{
		let name = &entry["name"].as_str().unwrap();
		let entry_properties = &entry["schema"].get("properties").unwrap();

		let mut props: Vec<GraphQLProperty> = Vec::new();

		for prop in entry_properties.as_object().unwrap().iter()
		{
			let prop_name = prop.0.clone();

			let raw_json_type: RawJsonType = prop.1["type"].as_str().unwrap().into();
			let json_type = JsonType {
				raw: raw_json_type,
				child: parse_json_type_child(prop.1, raw_json_type, true)
			};
			let scalar_type: ScalarType = json_type.into();

			props.push(GraphQLProperty {
				name: prop_name,
				scalar_type
			});
		}

		let object = format!(
			"type {} {{\n{}\n}}\n",
			format!("{}{}", (&name[..1].to_string()).to_uppercase(), &name[1..]),
			get_type_body(props)
		);

		sdl.push_str(&object);
		// println!("{:?}", entry);
	}

	println!("----- SDL GENERATED -----");
	println!("SDL: {}", sdl);
}

fn parse_json_type_child(json_data: &Value, json_type: RawJsonType, first: bool) -> Option<RawJsonType> {
	match json_type {
		RawJsonType::Array => {
			let items_type: RawJsonType = json_data["items"]["type"].as_str().unwrap().into();

			parse_json_type_child(&json_data["items"], items_type, false)
		},
		_ if first => None,
		_ => Some(json_type)
	}
}

fn get_type_body(props: Vec<GraphQLProperty>) -> String {
	let mut body = String::new();

	for prop in props {
		body.push_str(format!("\t{}: {}\n", prop.name, parse_graphql_prop_type(prop.scalar_type)).as_str())
	}

	body
}

fn parse_graphql_prop_type(prop_type: ScalarType) -> String {
	match prop_type.raw {
		RawScalarType::String => "String!",
		RawScalarType::Object => "String!",
		RawScalarType::Float => "Float!",
		RawScalarType::Int => "Int!",
		RawScalarType::Boolean => "Boolean!",
		RawScalarType::Array => {
			let mut str_type = String::new();

			str_type.push_str("[");
			str_type.push_str(parse_graphql_prop_type(
				prop_type
			).as_str());
			str_type.push_str("]!");

			str_type.as_str()
		}
		RawScalarType::Nullable => {
			"null"
		}
		RawScalarType::Enum => {
			"enum"
		}
	}.to_string()
}