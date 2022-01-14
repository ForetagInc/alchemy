use std::collections::HashMap;

use crate::lib::database::Database;
use super::document_property::SchemaDocumentProperty;

use serde_json::{ json, to_value as JsonValue };
use arangors_lite::{ AqlQuery };
use anyhow::Error;

pub struct Collection
{
	pub name: String,
	pub schema: Vec<SchemaDocumentProperty>,

	arango: Database
}

impl Collection
{
	pub async fn create_document(&self) -> Result<(), Error>
	{
		let properties = json!({
			"test": "test"
		});

		let aql = AqlQuery::builder()
			.query("INSERT @document INTO @@collection
				LET result = NEW RETURN result")
			.bind_var("@collection", self.name.clone())
			.bind_var("document", JsonValue(properties).unwrap())
			.build();

		let result = &self
			.arango
			.database
			.aql_query(aql)
			.await
			.unwrap();

		Ok(())
	}

	pub async fn read_document()
	{
		todo!()
	}

	pub async fn update_document()
	{
		todo!()
	}

	pub async fn delete_document()
	{
		todo!()
	}
}