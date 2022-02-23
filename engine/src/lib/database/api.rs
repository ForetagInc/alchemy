use serde_json::Value;

use crate::lib::schema::get_all_entries;

const ERR_CHILD_NOT_DEFINED: &str = "ERROR: Child type not defined";

struct GraphQLProperty {
	name: String,
	scalar_type: ScalarType,
	required: bool
}

#[derive(PartialEq)]
pub enum ScalarType {
	Array(Box<ScalarType>),
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
			JsonType::Array(value) => ScalarType::Array(Box::new((*value).into())),
			JsonType::Enum => ScalarType::Enum,
			JsonType::Boolean => ScalarType::Boolean,
			JsonType::Integer => ScalarType::Int,
			JsonType::Number => ScalarType::Float,
			JsonType::Object => ScalarType::Object,
			JsonType::String => ScalarType::String,
		}
	}
}

#[derive(Clone)]
pub enum JsonType {
	Array(Box<JsonType>),
	Enum,
	Boolean,
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
		let entry_properties = entry["schema"].get("properties").unwrap();
		let entry_required_properties: Vec<String> = entry["schema"].get("required")
			.unwrap().as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();

		let mut props: Vec<GraphQLProperty> = Vec::new();

		for prop in entry_properties.as_object().unwrap().iter()
		{
			let prop_name = prop.0.clone();

			let json_type = build_json_type(prop.1);
			let scalar_type: ScalarType = json_type.into();

			props.push(GraphQLProperty {
				name: prop_name.clone(),
				scalar_type,
				required: entry_required_properties.contains(&prop_name)
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
		_ => JsonType::String // This is an unreachable condition
	}
}

fn get_type_body(props: Vec<GraphQLProperty>) -> String {
	let mut body = String::new();

	for prop in props {
		body.push_str(format!("\t{}: {}\n", prop.name, parse_graphql_prop_type(prop.scalar_type, !prop.required)).as_str())
	}

	body
}

fn parse_graphql_prop_type(prop_type: ScalarType, nullable: bool) -> String {
	fn with_nullablity(prop_type: &str, nullable: bool) -> String {
		format!("{}{}", prop_type, if nullable { "" } else { "!" })
	}

	match prop_type {
		ScalarType::String => with_nullablity("String", nullable),
		ScalarType::Object => with_nullablity("String", nullable),
		ScalarType::Float => with_nullablity("Float", nullable),
		ScalarType::Int => with_nullablity("Int", nullable),
		ScalarType::Boolean => with_nullablity("Boolean", nullable),
		ScalarType::Array(value) => {
			let mut str_type = String::new();

			str_type.push_str("[");
			str_type.push_str(parse_graphql_prop_type(
				*value, nullable
			).as_str());
			str_type.push_str("]");

			with_nullablity(str_type.as_str(), nullable)
		}
		ScalarType::Enum => {
			"enum".to_string()
		}
	}
}