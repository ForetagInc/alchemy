pub mod arango;

pub mod schema;
// pub mod collection;

pub mod database;
pub use database::ArangoDB;
pub use database::DATABASE;