use serde_json::Value;
use std::collections::HashMap;

pub struct AQLQuery<'a> {
	pub properties: Vec<AQLProperty>,
	pub filter: Option<Box<dyn AQLNode>>,
	pub parameters: HashMap<&'a str, Value>,
	pub limit: u32,

	pub id: u32,
}

impl<'a> AQLQuery<'a> {
	pub fn new(id: u32) -> Box<AQLQuery<'a>> {
		Box::new(AQLQuery {
			properties: Vec::new(),
			filter: None,
			parameters: HashMap::new(),
			limit: 0,
			id
		})
	}

	pub fn to_aql(&self) -> String {
		format!(
			"
		FOR i_{} IN @@collection
			{}
				LIMIT {}
		RETURN {}",
			self.id,
			self.describe_filter(),
			self.limit,
			self.describe_parameters()
		)
	}

	pub fn describe_parameters(&self) -> String {
		format!(
			"{{
			{}
		}}",
			self.properties
				.iter()
				.map(|p| format!("\"{name}\": i_{}.`{name}`", self.id, name = p.name))
				.collect::<Vec<String>>()
				.join(",\n")
		)
	}

	fn describe_filter(&self) -> String {
		return if let Some(f) = &self.filter {
			format!("FILTER {}", f.describe(self.id))
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

impl<'a> AQLNode for AQLQueryBind<'a> {
	fn describe(&self, id: u32) -> String {
		format!("@arg_{}_{}", id, self.0)
	}
}

impl AQLNode for AQLQueryParameter {
	fn describe(&self, id: u32) -> String {
		format!("i_{}.`{}`", id, self.0)
	}
}
