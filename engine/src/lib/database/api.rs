use convert_case::Casing;
use serde_json::Value;
use std::fmt::Formatter;
use std::ops::Deref;

use crate::lib::schema::get_all_entries;

const ERR_CHILD_NOT_DEFINED: &str = "ERROR: Child type not defined";
const ERR_UNDEFINED_TYPE: &str = "ERROR: Undefined associated SDL type";

#[derive(Clone)]
pub struct GraphQLMap(pub Vec<GraphQLPrimitive>);

impl std::fmt::Display for GraphQLMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for primitive in self.0.iter() {
            write!(f, "{}", primitive)?
        }

        Ok(())
    }
}

#[derive(Clone)]
pub enum GraphQLPrimitive {
    Type(Box<GraphQLType>),
    Enum(Box<GraphQLEnum>),
}

#[derive(juniper::GraphQLInputObject)]
struct Coordinate {
    latitude: f64,
    longitude: f64,
}

impl std::fmt::Display for GraphQLPrimitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphQLPrimitive::Type(t) => {
                let object = format!("type {} {{\n{}}}\n", t.name, get_type_body(&t.properties));

                write!(f, "{}", object)
            }
            GraphQLPrimitive::Enum(e) => {
                let object = format!("enum {} {{\n{}\n}}\n", e.name, e.properties.join("\n"));

                write!(f, "{}", object)
            }
        }
    }
}

#[derive(Clone)]
pub struct GraphQLEnum {
    name: String,
    properties: Vec<String>,
}

#[derive(Clone)]
pub struct GraphQLType {
    pub name: String,
    pub properties: Vec<GraphQLProperty>,
}

#[derive(Default, Clone)]
pub struct GraphQLProperty {
    pub name: String,
    pub associated_type: Option<String>,
    pub scalar_type: ScalarType,
    pub required: bool,
}

#[derive(PartialEq, Default, Clone)]
pub enum ScalarType {
    Array(Box<ScalarType>),
    Enum(Vec<String>),
    #[default]
    String,
    Object,
    Float,
    Int,
    Boolean,
}

impl From<JsonType> for ScalarType {
    fn from(raw_type: JsonType) -> Self {
        match raw_type {
            JsonType::Array(value) => ScalarType::Array(Box::new((*value).into())),
            JsonType::Enum(values) => ScalarType::Enum(values),
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
    Enum(Vec<String>),
    Boolean,
    Integer,
    Number,
    Object,
    String,
}

pub async fn generate_sdl() -> GraphQLMap {
    let entries = get_all_entries().await;

    let mut sdl: GraphQLMap = GraphQLMap(Vec::new());

    println!("----- SDL GENERATION -----");

    let time = std::time::Instant::now();

    for entry in entries.clone().iter() {
        let type_name = pluralizer::pluralize(
            entry["name"]
                .as_str()
                .unwrap()
                .to_case(convert_case::Case::Pascal)
                .as_str(),
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

        let mut props: Vec<GraphQLProperty> = Vec::new();

        for prop in entry_properties.as_object().unwrap().iter() {
            let prop_name = prop.0.clone();

            let json_type = build_json_type(prop.1);
            let scalar_type: ScalarType = json_type.clone().into();

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

                sdl.0.push(GraphQLPrimitive::Enum(Box::new(GraphQLEnum {
                    name: enum_name,
                    properties: enum_values,
                })));
            }

            props.push(GraphQLProperty {
                name: prop_name.clone(),
                associated_type,
                scalar_type,
                required: entry_required_properties.contains(&prop_name),
                ..Default::default()
            });
        }

        sdl.0.push(GraphQLPrimitive::Type(Box::new(GraphQLType {
            name: type_name,
            properties: props,
        })))
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

fn get_type_body(props: &Vec<GraphQLProperty>) -> String {
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
    prop_type: &ScalarType,
    nullable: bool,
    associated_type: &Option<String>,
) -> String {
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
            str_type
                .push_str(parse_graphql_prop_type(value.deref(), true, associated_type).as_str());
            str_type.push_str("]");

            with_nullablity(str_type.as_str(), nullable)
        }
        ScalarType::Enum(_) if associated_type.is_some() => {
            with_nullablity(associated_type.clone().unwrap().as_str(), nullable)
        }
        _ => panic!("{}", ERR_UNDEFINED_TYPE),
    }
}
