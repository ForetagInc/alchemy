use serde_json::Value;
use std::collections::HashMap;

use crate::lib::database::api::DbRelationshipDirection;

pub struct AQLQueryRelationship {
	pub edge: String,
	pub direction: DbRelationshipDirection,
	pub variable_name: String,
}

pub struct AQLQuery<'a> {
	pub properties: Vec<AQLProperty>,
	pub filter: Option<Box<dyn AQLNode>>,
	pub parameters: HashMap<&'a str, Value>,
	pub relations: HashMap<String, AQLQuery<'a>>,
	pub limit: Option<i32>,
	pub relationship: Option<AQLQueryRelationship>,

	pub id: u32,
}

impl<'a> AQLQuery<'a> {
	pub fn new(id: u32) -> AQLQuery<'a> {
		AQLQuery {
			properties: Vec::new(),
			filter: None,
			parameters: HashMap::new(),
			relations: HashMap::new(),
			limit: None,
			relationship: None,
			id,
		}
	}

	pub fn to_aql(&self) -> String {
		if let Some(ref r) = self.relationship {
			format!(
				"FOR {} IN {} {} {} {} {} RETURN {}",
				self.get_variable_name(),
				r.direction.to_string(),
				r.variable_name,
				r.edge,
				self.describe_filter(),
				self.describe_limit(),
				self.describe_parameters()
			)
		} else {
			format!(
				"FOR {} IN @@collection {} {} RETURN {}",
				self.get_variable_name(),
				self.describe_filter(),
				self.describe_limit(),
				self.describe_parameters()
			)
		}
	}

	pub fn describe_parameters(&self) -> String {
		format!(
			"{{{}}}",
			self.properties
				.iter()
				.map(|p| format!(
					"\"{name}\": {}.`{name}`",
					self.get_variable_name(),
					name = p.name
				))
				.chain(self.relations.iter().map(|(key, query)| format!(
					"\"{}\": ({})",
					key,
					query.to_aql()
				)))
				.collect::<Vec<String>>()
				.join(",")
		)
	}

	fn describe_limit(&self) -> String {
		if let Some(limit) = self.limit {
			format!("LIMIT {}", limit)
		} else {
			"".to_string()
		}
	}

	fn describe_filter(&self) -> String {
		if let Some(f) = &self.filter {
			format!("FILTER {}", f.describe(self.id))
		} else {
			"".to_string()
		}
	}

	pub fn get_argument_key(&self, name: &str) -> String {
		format!("arg_{}_{}", self.id, name)
	}

	pub fn get_variable_name(&self) -> String {
		format!("i_{}", self.id)
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

impl From<&str> for AQLOperation {
	fn from(str: &str) -> Self {
		match str {
			"_eq" => Self::EQUAL,
			&_ => unreachable!(),
		}
	}
}

impl ToString for AQLOperation {
	fn to_string(&self) -> String {
		return match self {
			AQLOperation::EQUAL => "==".to_string(),
		};
	}
}

pub enum AQLLogicalOperator {
	AND,
	OR,
	NOT,
}

impl ToString for AQLLogicalOperator {
	fn to_string(&self) -> String {
		return match self {
			AQLLogicalOperator::AND => "AND".to_string(),
			AQLLogicalOperator::OR => "OR".to_string(),
			AQLLogicalOperator::NOT => "NOT".to_string(),
		};
	}
}

pub struct AQLLogicalFilter {
	pub nodes: Vec<Box<dyn AQLNode>>,
	pub operation: AQLLogicalOperator,
}

pub struct AQLQueryBind(pub String);
pub struct AQLQueryParameter(pub String);
pub struct AQLQueryValue(pub String);

pub trait AQLNode {
	fn describe(&self, id: u32) -> String;
}

impl AQLNode for AQLFilter {
	fn describe(&self, id: u32) -> String {
		format!(
			"({} {} {})",
			self.left_node.describe(id),
			self.operation.to_string(),
			self.right_node.describe(id)
		)
	}
}

impl AQLNode for AQLLogicalFilter {
	fn describe(&self, id: u32) -> String {
		format!(
			"({})",
			self.nodes
				.iter()
				.map(|n| format!("({})", n.describe(id)))
				.collect::<Vec<String>>()
				.join(&self.operation.to_string())
		)
	}
}

impl AQLNode for AQLQueryBind {
	fn describe(&self, id: u32) -> String {
		format!("@arg_{}_{}", id, self.0)
	}
}

impl AQLNode for AQLQueryParameter {
	fn describe(&self, id: u32) -> String {
		format!("i_{}.`{}`", id, self.0)
	}
}

impl AQLNode for AQLQueryValue {
	fn describe(&self, _: u32) -> String {
		format!("{:?}", self.0)
	}
}
