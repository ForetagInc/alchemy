use convert_case::Casing;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::Arc;

use crate::lib::schema::{get_all_collections, get_all_edges};

const ERR_CHILD_NOT_DEFINED: &str = "ERROR: Child type not defined";
const ERR_UNDEFINED_TYPE: &str = "ERROR: Undefined associated SDL type";

#[derive(Clone)]
pub struct DbMap {
	pub primitives: Vec<DbPrimitive>,
	pub relationships: Vec<DbRelationship>,
}

impl DbMap {
	pub fn new() -> Self {
		Self {
			primitives: Vec::new(),
			relationships: Vec::new(),
		}
	}
}

impl std::fmt::Display for DbMap {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for primitive in self.primitives.iter() {
			write!(f, "{}", primitive)?
		}

		Ok(())
	}
}

#[derive(Clone)]
pub enum DbPrimitive {
	Entity(Arc<DbEntity>),
	Enum(Arc<DbEnum>),
}

impl std::fmt::Display for DbPrimitive {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DbPrimitive::Entity(t) => {
				let object = format!("type {} {{\n{}}}\n", t.name, get_type_body(&t.properties));

				write!(f, "{}", object)
			}
			DbPrimitive::Enum(e) => {
				let object = format!("enum {} {{\n{}\n}}\n", e.name, e.properties.join("\n"));

				write!(f, "{}", object)
			}
		}
	}
}

#[derive(Clone)]
pub struct DbEnum {
	pub name: String,
	pub properties: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DbRelationshipType {
	OneToOne,
	OneToMany,
	ManyToMany,
}

impl From<&str> for DbRelationshipType {
	fn from(value: &str) -> Self {
		return match value {
			"one_to_one" => Self::OneToOne,
			"one_to_many" => Self::OneToMany,
			"many_to_many" => Self::ManyToMany,
			&_ => unreachable!(),
		};
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum DbRelationshipDirection {
	Inbound,
	Outbound,
	Any,
}

impl ToString for DbRelationshipDirection {
	fn to_string(&self) -> String {
		match *self {
			DbRelationshipDirection::Inbound => "INBOUND",
			DbRelationshipDirection::Outbound => "OUTBOUND",
			DbRelationshipDirection::Any => "ANY",
		}
		.to_string()
	}
}

impl From<&str> for DbRelationshipDirection {
	fn from(value: &str) -> Self {
		return match value {
			"outbound" => Self::Inbound,
			"inbound" => Self::Outbound,
			"any" => Self::Any,
			&_ => unreachable!(),
		};
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct DbRelationship {
	pub name: String,
	pub edge: String,
	pub from: Arc<DbEntity>,
	pub to: Arc<DbEntity>,
	pub relationship_type: DbRelationshipType,
	pub direction: DbRelationshipDirection,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DbEntity {
	pub name: String,
	pub collection_name: String,
	pub properties: Vec<DbProperty>,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct DbProperty {
	pub name: String,
	pub associated_type: Option<String>,
	pub scalar_type: DbScalarType,
	pub required: bool,
}

#[derive(PartialEq, Default, Clone, Debug)]
pub enum DbScalarType {
	Array(Box<DbScalarType>),
	Enum(Vec<String>),
	#[default]
	String,
	Object,
	Float,
	Int,
	Boolean,
}

impl From<JsonType> for DbScalarType {
	fn from(raw_type: JsonType) -> Self {
		match raw_type {
			JsonType::Array(value) => DbScalarType::Array(Box::new((*value).into())),
			JsonType::Enum(values) => DbScalarType::Enum(values),
			JsonType::Boolean => DbScalarType::Boolean,
			JsonType::Integer => DbScalarType::Int,
			JsonType::Number => DbScalarType::Float,
			JsonType::Object => DbScalarType::Object,
			JsonType::String => DbScalarType::String,
		}
	}
}

#[derive(Clone)]
pub enum JsonType {
	Array(Box<JsonType>),
	Enum(Vec<String>),
	Boolean,
	Integer,
	Number,
	Object,
	String,
}

pub async fn generate_sdl() -> DbMap {
	let collections = get_all_collections().await;
	let edges = get_all_edges().await;

	let mut sdl: DbMap = DbMap::new();
	let mut collections_by_keys: HashMap<String, Arc<DbEntity>> = HashMap::new();

	println!("----- SDL GENERATION -----");

	let time = std::time::Instant::now();

	for entry in collections.clone().iter() {
		let collection_name = entry["name"].as_str().unwrap().to_string();

		let type_name = pluralizer::pluralize(
			collection_name.to_case(convert_case::Case::Pascal).as_str(),
			1,
			false,
		);
		let entry_properties = entry["schema"].get("properties").unwrap();
		let entry_required_properties: Vec<String> = entry["schema"]
			.get("required")
			.unwrap()
			.as_array()
			.unwrap()
			.iter()
			.map(|v| v.as_str().unwrap().to_string())
			.collect();

		let mut props: Vec<DbProperty> = Vec::new();

		for prop in entry_properties.as_object().unwrap().iter() {
			let prop_name = prop.0.clone();

			let json_type = build_json_type(prop.1);
			let scalar_type: DbScalarType = json_type.clone().into();

			let mut associated_type: Option<String> = None;

			if let JsonType::Enum(values) = json_type {
				let enum_values: Vec<String> = values
					.iter()
					.map(|v| format!("\t{}", v.to_case(convert_case::Case::UpperSnake)))
					.collect();

				let enum_name = format!(
					"{}{}Enum",
					type_name,
					prop_name.to_case(convert_case::Case::Pascal)
				);

				associated_type = Some(enum_name.clone());

				sdl.primitives.push(DbPrimitive::Enum(Arc::new(DbEnum {
					name: enum_name,
					properties: enum_values,
				})));
			}

			props.push(DbProperty {
				name: prop_name.clone(),
				associated_type,
				scalar_type,
				required: entry_required_properties.contains(&prop_name),
				..Default::default()
			});
		}

		let entity = Arc::new(DbEntity {
			name: type_name,
			collection_name: collection_name.clone(),
			properties: props,
		});

		// We insert it on this hash map for future use of relationships
		collections_by_keys.insert(collection_name, entity.clone());

		sdl.primitives.push(DbPrimitive::Entity(entity.clone()))
	}

	for entry in edges.clone().iter() {
		let prop_name = entry["name"].as_str().unwrap();
		let edge = entry["edge"].as_str().unwrap();
		let from = entry["from"].as_str().unwrap();
		let to = entry["to"].as_str().unwrap();
		let relationship_type: DbRelationshipType = entry["type"].as_str().unwrap().into();
		let relationship_direction: DbRelationshipDirection =
			entry["direction"].as_str().unwrap().into();

		if let (Some(from_entity), Some(to_entity)) =
			(collections_by_keys.get(from), collections_by_keys.get(to))
		{
			sdl.relationships.push(DbRelationship {
				name: prop_name.to_string(),
				edge: edge.to_string(),
				from: from_entity.clone(),
				to: to_entity.clone(),
				relationship_type,
				direction: relationship_direction,
			})
		}
	}

	println!("----- SDL GENERATED in {:?} -----", time.elapsed());
	println!("{}", sdl);

	sdl
}

fn build_json_type(json_data: &Value) -> JsonType {
	if let Some(enum_data) = json_data["enum"].as_array() {
		return JsonType::Enum(
			enum_data
				.iter()
				.map(|v| v.as_str().unwrap().to_string())
				.collect(),
		);
	}

	let data_type = json_data["type"].as_str().unwrap();

	match data_type {
		"array" => JsonType::Array(Box::new(build_json_type(&json_data["items"]))),
		"boolean" => JsonType::Boolean,
		"integer" => JsonType::Integer,
		"number" => JsonType::Number,
		"object" => JsonType::Object,
		"string" => JsonType::String,
		_ => JsonType::String, // This is an unreachable condition
	}
}

fn get_type_body(props: &Vec<DbProperty>) -> String {
	let mut body = String::new();

	for prop in props {
		body.push_str(
			format!(
				"\t{}: {}\n",
				prop.name,
				parse_graphql_prop_type(&prop.scalar_type, !prop.required, &prop.associated_type)
			)
			.as_str(),
		)
	}

	body
}

fn parse_graphql_prop_type(
	prop_type: &DbScalarType,
	nullable: bool,
	associated_type: &Option<String>,
) -> String {
	fn with_nullablity(prop_type: &str, nullable: bool) -> String {
		format!("{}{}", prop_type, if nullable { "" } else { "!" })
	}

	match prop_type {
		DbScalarType::String => with_nullablity("String", nullable),
		DbScalarType::Object => with_nullablity("String", nullable),
		DbScalarType::Float => with_nullablity("Float", nullable),
		DbScalarType::Int => with_nullablity("Int", nullable),
		DbScalarType::Boolean => with_nullablity("Boolean", nullable),
		DbScalarType::Array(value) => {
			let mut str_type = String::new();

			str_type.push_str("[");
			str_type
				.push_str(parse_graphql_prop_type(value.deref(), true, associated_type).as_str());
			str_type.push_str("]");

			with_nullablity(str_type.as_str(), nullable)
		}
		DbScalarType::Enum(_) if associated_type.is_some() => {
			with_nullablity(associated_type.clone().unwrap().as_str(), nullable)
		}
		_ => panic!("{}", ERR_UNDEFINED_TYPE),
	}
}
