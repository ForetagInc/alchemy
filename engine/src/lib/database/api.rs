use crate::lib::schema::get_all_entries;

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
			RawJsonType::Array => Self {
				raw: json_type.raw.into(),
				child: Some(json_type.child.expect("ERROR: Array child type not defined").into())
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

pub enum RawJsonType {
	Array,
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
			"boolean" => Self::Boolean,
			"integer" => Self::Integer,
			"number" => Self::Number,
			"object" => Self::Object,
			"string" => Self::String,
			_ => Self::Nullable
		}
	}
}

impl From<&str> for JsonType {
	fn from(s: &str) -> Self {
		match s {
			_ => Self {
				raw: s.into(),
				child: None
			}
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

			let json_type: JsonType = prop.1["type"].as_str().unwrap().into();
			let scalar_type: ScalarType = json_type.into();

			props.push(GraphQLProperty {
				name: prop_name,
				scalar_type
			});
		}

		let object = format!(
			"type {} {{\n\t{}\n}}\n",
			format!("{}{}", (&name[..1].to_string()).to_uppercase(), &name[1..]),
			get_type_body(props)
		);

		sdl.push_str(&object);
		// println!("{:?}", entry);
	}

	println!("----- SDL GENERATED -----");
	println!("SDL: {}", sdl);
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
		RawScalarType::String => "String",
		RawScalarType::Object => "String",
		RawScalarType::Float => "Float",
		RawScalarType::Int => "Int",
		RawScalarType::Boolean => "Boolean",
		RawScalarType::Array => {
			"array"
		}
		RawScalarType::Nullable => {
			"null"
		}
		RawScalarType::Enum => {
			"enum"
		}
	}.to_string()
}