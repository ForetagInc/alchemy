pub mod arango;

pub mod schema;
// pub mod collection;

mod api;
pub use api::generate_sdl;

pub mod database;
pub use database::ArangoDB;
pub use database::DATABASE;