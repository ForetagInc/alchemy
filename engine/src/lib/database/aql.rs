use serde_json::Value;
use std::collections::HashMap;

use crate::lib::database::api::DbEntity;

pub struct AQLQuery<'a> {
	pub properties: Vec<AQLProperty>,
	pub filter: Option<Box<dyn AQLNode>>,
	pub parameters: HashMap<&'a str, Value>,
	pub limit: u32,
}

impl<'a> AQLQuery<'a> {
	pub fn new() -> Box<AQLQuery<'a>> {
		Box::new(AQLQuery {
			properties: Vec::new(),
			filter: None,
			parameters: HashMap::new(),
			limit: 0,
		})
	}

	pub fn to_aql(&self) -> String {
		format!(
			"
		FOR a IN @@collection
			{}
				LIMIT {}
		RETURN {}",
			self.describe_filter(),
			self.limit,
			self.describe_parameters("a")
		)
	}

	pub fn describe_parameters(&self, item_query_var: &str) -> String {
		format!(
			"{{
			{}
		}}",
			self.properties
				.iter()
				.map(|p| format!("\"{name}\": {}.`{name}`", item_query_var, name = p.name))
				.collect::<Vec<String>>()
				.join(",\n")
		)
	}

	fn describe_filter(&self) -> String {
		return if let Some(f) = &self.filter {
			format!("FILTER {}", f.describe())
		} else {
			"".to_string()
		}
	}
}

unsafe impl<'a> Send for AQLQuery<'a> {}

#[derive(Debug)]
pub struct AQLProperty {
	pub name: String,
}

pub struct AQLFilter {
	pub left_node: Box<dyn AQLNode>,
	pub operation: AQLOperation,
	pub right_node: Box<dyn AQLNode>,
}

pub enum AQLOperation {
	EQUAL,
}

impl ToString for AQLOperation {
	fn to_string(&self) -> String {
		return match self {
			AQLOperation::EQUAL => "==".to_string(),
		};
	}
}

pub struct AQLQueryBind<'a>(pub &'a str);
pub struct AQLQueryParameter(pub String);

pub trait AQLNode {
	fn describe(&self) -> String;
}

impl AQLNode for AQLFilter {
	fn describe(&self) -> String {
		format!(
			"({} {} {})",
			self.left_node.describe(),
			self.operation.to_string(),
			self.right_node.describe()
		)
	}
}

impl<'a> AQLNode for AQLQueryBind<'a> {
	fn describe(&self) -> String {
		format!("@{}", self.0)
	}
}

impl AQLNode for AQLQueryParameter {
	fn describe(&self) -> String {
		self.0.to_string()
	}
}
