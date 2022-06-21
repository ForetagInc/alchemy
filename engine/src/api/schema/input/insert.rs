use std::collections::HashMap;
use std::marker::PhantomData;

use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry};

use crate::api::schema::input::filter::{
	parse_indices_attributes, EntityIndicesFilter, EntityIndicesFilterData,
};
use crate::api::schema::operations::{OperationData, OperationRegistry};
use crate::api::schema::{build_argument_from_property, input_value_to_string, AsyncScalarValue};
use crate::lib::database::api::DbRelationship;

fn build_insert_arg_from_relationship<'r, S>(
	registry: &mut Registry<'r, S>,
	relationship: &DbRelationship,
	info: &EntityRelationshipInsertData<S>,
) -> Argument<'r, S>
where
	S: AsyncScalarValue,
{
	if relationship.relationship_type.returns_array() {
		registry.arg::<Option<Vec<EntityRelationshipInsert>>>(relationship.name.as_str(), info)
	} else {
		registry.arg::<Option<EntityRelationshipInsert>>(relationship.name.as_str(), info)
	}
}

#[derive(Debug)]
pub enum EntityInsertRelationship<S>
where
	S: AsyncScalarValue,
{
	Existing(String, HashMap<String, InputValue<S>>),
	New(String, InputValue<S>),
}

pub struct EntityInsert<'a, S>
where
	S: AsyncScalarValue,
{
	pub attributes: String,
	pub relationships: Vec<EntityInsertRelationship<S>>,

	_marker: PhantomData<&'a ()>,
}

pub struct EntityInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub name: String,
	pub data: &'a OperationData<S>,
	pub registry: &'a OperationRegistry<S>,
}

impl<'a, S> EntityInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub fn new(data: &'a OperationData<S>, registry: &'a OperationRegistry<S>) -> Self {
		Self {
			name: format!("{}Insert", data.entity.name.as_str()),
			data,
			registry,
		}
	}
}

impl<'a, S> GraphQLType<S> for EntityInsert<'a, S>
where
	S: AsyncScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut args = Vec::new();

		let attributes = registry.arg::<EntityAttributesInsert>(
			"attributes",
			&EntityAttributesInsertData::new(info.data, info.registry),
		);

		args.push(attributes);

		if !info.data.relationships.is_empty() {
			let relationships = registry.arg::<Option<EntityRelationshipsInsert>>(
				"relationships",
				&EntityRelationshipsInsertData::new(info.data, info.registry),
			);

			args.push(relationships);
		}

		registry
			.build_input_object_type::<EntityInsert<S>>(info, &args)
			.into_meta()
	}
}

impl<'a, S> GraphQLValue<S> for EntityInsert<'a, S>
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = EntityInsertData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> FromInputValue<S> for EntityInsert<'a, S>
where
	S: AsyncScalarValue,
{
	fn from_input_value(data: &InputValue<S>) -> Option<Self> {
		let mut attributes = String::new();
		let mut relationships = Vec::new();

		for (k, object) in data.to_object_value().unwrap() {
			if k == "attributes" {
				attributes = input_value_to_string(object)
			} else if k == "relationships" {
				for (rel_key, rel_object) in object.to_object_value().unwrap() {
					for r in rel_object.to_list_value().unwrap() {
						for (rel_type, rel_data) in r.to_object_value().unwrap() {
							match rel_type {
								"addExisting" => {
									relationships.push(EntityInsertRelationship::Existing(
										rel_key.to_string(),
										parse_indices_attributes(rel_data),
									))
								}
								"addNew" => relationships.push(EntityInsertRelationship::New(
									rel_key.to_string(),
									rel_data.clone(),
								)),
								&_ => {}
							}
						}
					}
				}
			}
		}

		Some(Self {
			attributes,
			relationships,

			_marker: Default::default(),
		})
	}
}

pub struct EntityAttributesInsert<'a> {
	pub data: String,

	_marker: PhantomData<&'a ()>,
}

pub struct EntityAttributesInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub name: String,
	pub data: &'a OperationData<S>,
	pub registry: &'a OperationRegistry<S>,
}

impl<'a, S> EntityAttributesInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub fn new(data: &'a OperationData<S>, registry: &'a OperationRegistry<S>) -> Self {
		Self {
			name: format!("{}AttributesInsert", data.entity.name.as_str()),
			data,
			registry,
		}
	}
}

impl<'a, S> GraphQLType<S> for EntityAttributesInsert<'a>
where
	S: AsyncScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut args = Vec::new();

		for property in &info.data.entity.properties {
			if property.name.eq("_key") {
				continue;
			}

			let arg = build_argument_from_property(
				registry,
				&property,
				&property.scalar_type,
				property.required,
			);

			args.push(arg);
		}

		registry
			.build_input_object_type::<EntityAttributesInsert>(info, &args)
			.into_meta()
	}
}

impl<'a, S> GraphQLValue<S> for EntityAttributesInsert<'a>
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = EntityAttributesInsertData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> FromInputValue<S> for EntityAttributesInsert<'a>
where
	S: AsyncScalarValue,
{
	fn from_input_value(data: &InputValue<S>) -> Option<Self> {
		Some(Self {
			data: input_value_to_string(data),

			_marker: Default::default(),
		})
	}
}

pub struct EntityRelationshipsInsert<'a> {
	pub data: String,

	_marker: PhantomData<&'a ()>,
}

pub struct EntityRelationshipsInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub name: String,
	pub data: &'a OperationData<S>,
	pub registry: &'a OperationRegistry<S>,
}

impl<'a, S> EntityRelationshipsInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub fn new(data: &'a OperationData<S>, registry: &'a OperationRegistry<S>) -> Self {
		Self {
			name: format!("{}RelationshipsInsert", data.entity.name.as_str()),
			data,
			registry,
		}
	}
}

impl<'a, S> GraphQLType<S> for EntityRelationshipsInsert<'a>
where
	S: AsyncScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut args = Vec::new();

		for relationship in &*info.data.relationships {
			let rel_data = &info
				.registry
				.get_operation_data(&relationship.to.name)
				.expect("Relationship entity operation data not found");

			let rel_info = &EntityRelationshipInsertData::new(rel_data, &info.registry);

			let arg = build_insert_arg_from_relationship(registry, relationship, rel_info);

			args.push(arg);
		}

		registry
			.build_input_object_type::<EntityRelationshipsInsert>(info, &args)
			.into_meta()
	}
}

impl<'a, S> GraphQLValue<S> for EntityRelationshipsInsert<'a>
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = EntityRelationshipsInsertData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> FromInputValue<S> for EntityRelationshipsInsert<'a>
where
	S: AsyncScalarValue,
{
	fn from_input_value(data: &InputValue<S>) -> Option<Self> {
		Some(Self {
			data: input_value_to_string(data),

			_marker: Default::default(),
		})
	}
}

pub struct EntityRelationshipInsert<'a> {
	pub data: String,

	_marker: PhantomData<&'a ()>,
}

pub struct EntityRelationshipInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub name: String,
	pub data: &'a OperationData<S>,
	pub registry: &'a OperationRegistry<S>,
}

impl<'a, S> EntityRelationshipInsertData<'a, S>
where
	S: AsyncScalarValue,
{
	pub fn new(data: &'a OperationData<S>, registry: &'a OperationRegistry<S>) -> Self {
		Self {
			name: format!("{}RelationshipInsert", data.entity.name.as_str()),
			data,
			registry,
		}
	}
}

impl<'a, S> GraphQLType<S> for EntityRelationshipInsert<'a>
where
	S: AsyncScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let new = registry.arg::<Option<EntityAttributesInsert>>(
			"addNew",
			&EntityAttributesInsertData::new(info.data, info.registry),
		);
		let existing = registry.arg::<Option<EntityIndicesFilter<S>>>(
			"addExisting",
			&EntityIndicesFilterData::new(info.data),
		);

		registry
			.build_input_object_type::<EntityRelationshipInsert>(info, &vec![new, existing])
			.into_meta()
	}
}

impl<'a, S> GraphQLValue<S> for EntityRelationshipInsert<'a>
where
	S: AsyncScalarValue,
{
	type Context = ();
	type TypeInfo = EntityRelationshipInsertData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> FromInputValue<S> for EntityRelationshipInsert<'a>
where
	S: AsyncScalarValue,
{
	fn from_input_value(data: &InputValue<S>) -> Option<Self> {
		Some(Self {
			data: input_value_to_string(data),

			_marker: Default::default(),
		})
	}
}
