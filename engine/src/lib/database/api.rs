use serde_json::Value;

use crate::lib::schema::get_all_entries;

const ERR_CHILD_NOT_DEFINED: &str = "ERROR: Child type not defined";

struct GraphQLProperty {
	name: String,
	scalar_type: ScalarType
}

#[derive(PartialEq)]
pub enum ScalarType {
	Array(Box<ScalarType>),
	Nullable,
	String,
	Object,
	Float,
	Int,
	Boolean,
	Enum,
}

impl From<JsonType> for ScalarType {
	fn from(raw_type: JsonType) -> Self {
		match raw_type {
			JsonType::Array(a) => ScalarType::Array(Box::new((*a).into())),
			JsonType::Enum => ScalarType::Enum,
			JsonType::Boolean => ScalarType::Boolean,
			JsonType::Integer => ScalarType::Int,
			JsonType::Number => ScalarType::Float,
			JsonType::Object => ScalarType::Object,
			JsonType::String => ScalarType::String,
			JsonType::Nullable => ScalarType::Nullable
		}
	}
}

#[derive(Clone)]
pub enum JsonType {
	Array(Box<JsonType>),
	Enum,
	Boolean,
	Nullable,
	Integer,
	Number,
	Object,
	String,
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

			let json_type = build_json_type(prop.1);
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

fn build_json_type(json_data: &Value) -> JsonType {
	let data_type = json_data["type"].as_str().unwrap();

	match data_type {
		"array" => JsonType::Array(Box::new(build_json_type(&json_data["items"]))),
		"enum" => JsonType::Enum,
		"boolean" => JsonType::Boolean,
		"integer" => JsonType::Integer,
		"number" => JsonType::Number,
		"object" => JsonType::Object,
		"string" => JsonType::String,
		_ => JsonType::Nullable
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
		ScalarType::String => "String!".to_string(),
		ScalarType::Object => "String!".to_string(),
		ScalarType::Float => "Float!".to_string(),
		ScalarType::Int => "Int!".to_string(),
		ScalarType::Boolean => "Boolean!".to_string(),
		ScalarType::Array => {
			let mut str_type = String::new();

			str_type.push_str("[");
			str_type.push_str(parse_graphql_prop_type(
				prop_type.child
			).as_str());
			str_type.push_str("]!");

			str_type
		},
		ScalarType::Nullable => {
			"null".to_string()
		}
		ScalarType::Enum => {
			"enum".to_string()
		}
	}
}