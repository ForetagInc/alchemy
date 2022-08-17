use convert_case::Casing;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Formatter;
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

#[derive(Clone)]
pub enum DbPrimitive {
	Entity(Arc<DbEntity>),
	Enum(Arc<DbEnum>),
}

impl std::fmt::Display for DbPrimitive {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DbPrimitive::Entity(t) => {
				write!(f, "{}", t.name)
			}
			DbPrimitive::Enum(e) => {
				write!(f, "{}", e.name)
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
	ManyToOne,
}

impl DbRelationshipType {
	pub fn returns_array(&self) -> bool {
		match self {
			DbRelationshipType::OneToOne | DbRelationshipType::ManyToOne => false,
			DbRelationshipType::OneToMany | DbRelationshipType::ManyToMany => true,
		}
	}
}

impl From<&str> for DbRelationshipType {
	fn from(value: &str) -> Self {
		return match value {
			"one_to_one" => Self::OneToOne,
			"one_to_many" => Self::OneToMany,
			"many_to_many" => Self::ManyToMany,
			"many_to_one" => Self::ManyToOne,
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

		// Adding document key property to all entities
		props.push(DbProperty {
			name: "_key".to_string(),
			scalar_type: DbScalarType::Int,
			required: true,
			..Default::default()
		});

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

	println!("SDL generated in {:?}", time.elapsed());
	println!(
		"Found [{}] entities and [{}] relationships",
		sdl.primitives
			.iter()
			.map(|p| p.to_string())
			.collect::<Vec<String>>()
			.join(", "),
		sdl.relationships
			.iter()
			.map(|p| p.name.as_str())
			.collect::<Vec<&str>>()
			.join(", "),
	);

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
