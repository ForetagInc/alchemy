pub mod properties;
pub use properties::{ SchemaDocumentProperty, SchemaDocumentPropertyValues };

// pub mod generation;
pub mod entries;
pub use entries::{ 
	create_entry,
	delete_entry
};