use serde_json::Value;
use std::collections::HashMap;

use crate::lib::database::api::DbEntity;

pub struct AQLQuery<'a> {
	pub properties: Vec<AQLProperty>,
	pub filter: Box<dyn AQLNode>,
	pub parameters: HashMap<&'a str, Value>,
}

impl<'a> AQLQuery<'a> {
	pub fn new(filter: Box<dyn AQLNode>) -> Box<AQLQuery<'a>> {
		Box::new(AQLQuery {
			filter,
			properties: Vec::new(),
			parameters: HashMap::new(),
		})
	}

	pub fn to_aql(&self, limit: u32) -> String {
		format!(
			"
		FOR a IN @@collection
			FILTER {}
				LIMIT {}
		RETURN {}",
			self.filter.describe(),
			limit,
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
