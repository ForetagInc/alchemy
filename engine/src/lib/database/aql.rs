use std::collections::HashMap;

use crate::lib::database::api::{DbRelationshipDirection, DbRelationshipType};

pub struct AQLQueryRelationship {
	pub edge: String,
	pub direction: DbRelationshipDirection,
	pub relationship_type: DbRelationshipType,
	pub variable_name: String,
}

pub enum AQLQueryMethod {
	Get,
	Update,
}

pub struct AQLQuery {
	pub properties: Vec<AQLProperty>,
	pub method: AQLQueryMethod,
	pub filter: Option<Box<dyn AQLNode>>,
	pub relations: HashMap<String, AQLQuery>,
	pub updates: String,
	pub limit: Option<i32>,
	pub relationship: Option<AQLQueryRelationship>,

	pub id: u32,
}

impl AQLQuery {
	pub fn new(id: u32) -> AQLQuery {
		AQLQuery {
			properties: Vec::new(),
			method: AQLQueryMethod::Get,
			filter: None,
			relations: HashMap::new(),
			updates: "null".to_string(),
			limit: None,
			relationship: None,
			id,
		}
	}

	pub fn to_aql(&self) -> String {
		match self.method {
			AQLQueryMethod::Get => self.to_get_aql(),
			AQLQueryMethod::Update => self.to_update_aql(),
		}
	}

	pub fn describe_parameters(&self) -> String {
		let variable = match self.method {
			AQLQueryMethod::Get => self.get_variable_name(),
			AQLQueryMethod::Update => "NEW".to_string(),
		};

		format!(
			"{{{}}}",
			self.properties
				.iter()
				.map(|p| format!("\"{name}\": {}.`{name}`", variable, name = p.name))
				.chain(self.relations.iter().map(|(key, query)| format!(
					"\"{}\": {}",
					key,
					query.to_aql()
				)))
				.collect::<Vec<String>>()
				.join(",")
		)
	}

	fn to_get_aql(&self) -> String {
		if let Some(ref r) = self.relationship {
			format!(
				"(FOR {} IN {} {} {} {} {} RETURN {}){}",
				self.get_variable_name(),
				r.direction.to_string(),
				r.variable_name,
				r.edge,
				self.describe_filter(),
				self.describe_limit(),
				self.describe_parameters(),
				if !r.relationship_type.returns_array() {
					"[0]"
				} else {
					""
				}
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

	fn to_update_aql(&self) -> String {
		format!(
			"FOR {var} IN @@collection {} UPDATE {var}.`_key` WITH {} IN @@collection RETURN {}",
			self.describe_filter(),
			self.updates,
			self.describe_parameters(),
			var = self.get_variable_name(),
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

unsafe impl Send for AQLQuery {}

#[derive(Debug)]
pub struct AQLProperty {
	pub name: String,
}

pub struct AQLFilterOperation {
	pub left_node: Box<dyn AQLNode>,
	pub operation: AQLOperation,
	pub right_node: Box<dyn AQLNode>,
}

pub struct AQLFilter {
	pub attr_node: Box<dyn AQLNode>,
	pub and_node: Option<Box<dyn AQLNode>>,
	pub or_node: Option<Box<dyn AQLNode>>,
	pub not_node: Option<Box<dyn AQLNode>>,
}

pub enum AQLOperation {
	Equal,
	GreaterThan,
	GreaterOrEqualThan,
	Like,
	LessThan,
	LessOrEqualThan,
	NotEqual,
	NotLike,
	NotRegex,
	Regex,
}

impl From<&str> for AQLOperation {
	fn from(str: &str) -> Self {
		match str {
			"_eq" => Self::Equal,
			"_gt" => Self::GreaterThan,
			"_gte" => Self::GreaterOrEqualThan,
			"_like" => Self::Like,
			"_lt" => Self::LessThan,
			"_lte" => Self::LessOrEqualThan,
			"_neq" => Self::NotEqual,
			"_nlike" => Self::NotLike,
			"_nregex" => Self::NotRegex,
			"_regex" => Self::Regex,
			&_ => unreachable!(),
		}
	}
}

impl ToString for AQLOperation {
	fn to_string(&self) -> String {
		return match self {
			AQLOperation::Equal => "==",
			AQLOperation::GreaterThan => ">",
			AQLOperation::GreaterOrEqualThan => ">=",
			AQLOperation::Like => "LIKE",
			AQLOperation::LessThan => "<",
			AQLOperation::LessOrEqualThan => "<=",
			AQLOperation::NotEqual => "!=",
			AQLOperation::NotLike => "NOT LIKE",
			AQLOperation::NotRegex => "=~",
			AQLOperation::Regex => "!~",
		}
		.to_string();
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
			AQLLogicalOperator::AND => " AND ".to_string(),
			AQLLogicalOperator::OR => " OR ".to_string(),
			AQLLogicalOperator::NOT => " AND NOT ".to_string(),
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
pub struct AQLQueryRaw(pub String);

pub trait AQLNode {
	fn describe(&self, id: u32) -> String;

	fn valid(&self) -> bool {
		true
	}
}

impl AQLNode for AQLFilterOperation {
	fn describe(&self, id: u32) -> String {
		format!(
			"({} {} {})",
			self.left_node.describe(id),
			self.operation.to_string(),
			self.right_node.describe(id)
		)
	}
}

impl AQLNode for AQLFilter {
	fn describe(&self, id: u32) -> String {
		let mut out = String::new();

		fn add_to_if_exists(
			mut str: String,
			node: &Option<Box<dyn AQLNode>>,
			operator: AQLLogicalOperator,
			id: u32,
		) -> String {
			if let Some(n) = node {
				str.push_str(operator.to_string().as_str());
				str.push_str(n.describe(id).as_str());
			}

			str
		}

		out.push_str(self.attr_node.describe(id).as_str());

		out = add_to_if_exists(out, &self.and_node, AQLLogicalOperator::AND, id);
		out = add_to_if_exists(out, &self.or_node, AQLLogicalOperator::OR, id);
		out = add_to_if_exists(out, &self.not_node, AQLLogicalOperator::NOT, id);

		out
	}

	fn valid(&self) -> bool {
		self.attr_node.valid()
	}
}

impl AQLNode for AQLLogicalFilter {
	fn describe(&self, id: u32) -> String {
		let mut str = String::new();

		if self.nodes.len() > 1 {
			str.push('(');
		}

		str.push_str(
			self.nodes
				.iter()
				.map(|n| format!("{}", n.describe(id)))
				.collect::<Vec<String>>()
				.join(&self.operation.to_string())
				.as_str(),
		);

		if self.nodes.len() > 1 {
			str.push(')');
		}

		str
	}

	fn valid(&self) -> bool {
		self.nodes.len() > 0
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

impl AQLNode for AQLQueryRaw {
	fn describe(&self, _: u32) -> String {
		self.0.clone()
	}
}
