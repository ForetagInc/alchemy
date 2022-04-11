use juniper::meta::{Argument, MetaType};
use juniper::{
	Arguments, FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry, ScalarValue,
};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::api::input::string_filter::{StringFilter, StringFilterData};
use crate::api::schema::operations::OperationData;
use crate::lib::database::api::DbScalarType;
use crate::lib::database::aql::{AQLLogicalFilter, AQLLogicalOperator, AQLNode};

pub trait FilterOperation<S>
where
	S: ScalarValue,
{
	fn get_schema_argument<'r, 'd>(registry: &mut Registry<'r, S>) -> Argument<'r, S>;
}

pub struct EntityFilterData<'a, S>
where
	S: ScalarValue,
{
	pub name: String,
	pub operation_data: &'a OperationData<S>,
}

#[derive(Debug)]
pub struct FilterAttributes<S>
where
	S: ScalarValue,
{
	pub attributes: HashMap<String, InputValue<S>>,
	pub and: Option<Vec<FilterAttributes<S>>>,
	pub not: Box<Option<FilterAttributes<S>>>,
	pub or: Option<Vec<FilterAttributes<S>>>,
}

#[derive(Debug)]
pub struct EntityFilter<'a, S>
where
	S: ScalarValue + 'a,
{
	pub filter_arguments: FilterAttributes<S>,

	_marker: PhantomData<&'a S>,
}

impl<'a, S> GraphQLValue<S> for EntityFilter<'a, S>
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = EntityFilterData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> GraphQLType<S> for EntityFilter<'a, S>
where
	S: ScalarValue,
{
	fn name(info: &Self::TypeInfo) -> Option<&str> {
		Some(info.name.as_str())
	}

	fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
	where
		S: 'r,
	{
		let mut args = Vec::new();

		let and = registry.arg::<Option<Vec<Self>>>("_and", info);
		let not = registry.arg::<Option<Self>>("_not", info);
		let or = registry.arg::<Option<Vec<Self>>>("_or", info);

		args.extend([and, not, or]);

		for property in &info.operation_data.entity.properties {
			let arg = match property.scalar_type {
				DbScalarType::String => registry.arg::<Option<StringFilter<'a, S>>>(
					property.name.as_str(),
					&StringFilterData::from(info),
				),
				_ => registry.arg::<Option<i32>>(property.name.as_str(), &()),
			};

			args.push(arg)
		}

		registry
			.build_input_object_type::<Self>(info, &args)
			.into_meta()
	}
}

impl<'a, S> FromInputValue<S> for EntityFilter<'a, S>
where
	S: ScalarValue,
{
	fn from_input_value(value: &InputValue<S>) -> Option<Self> {
		Some(Self {
			filter_arguments: parse_filter_attributes(value),

			_marker: Default::default(),
		})
	}
}

fn parse_filter_attributes<'a, S>(data: &InputValue<S>) -> FilterAttributes<S>
where
	S: ScalarValue + 'a,
{
	let mut attributes = HashMap::new();

	match data {
		InputValue::Object(items) => {
			let ignore_keys = vec!["_and", "_not", "_or"];

			for (key, value) in items {
				if ignore_keys.contains(&key.item.as_str()) {
					continue;
				}

				attributes.insert(key.item.clone(), value.item.clone());
			}
		}
		_ => {}
	}

	FilterAttributes {
		attributes,
		and: None,
		not: Box::new(None),
		or: None,
	}
}

pub fn get_aql_filter_from_args<S>(args: &Arguments<S>, data: &OperationData<S>) -> Box<dyn AQLNode>
where
	S: ScalarValue,
{
	let mut node = AQLLogicalFilter {
		nodes: Vec::new(),
		operation: AQLLogicalOperator::AND,
	};

	if let Some(entity_filter) = args.get::<EntityFilter<S>>("where") {
		let attributes = entity_filter.filter_arguments.attributes;

		let properties: HashMap<String, DbScalarType> = data
			.entity
			.properties
			.iter()
			.map(|p| (p.name.clone(), p.scalar_type.clone()))
			.collect();

		for (name, value) in attributes {
			if let Some(scalar) = properties.get(&name) {
				node.nodes.push(Box::new(create_aql_node_from_attribute(
					name, value, scalar,
				)));
			}
		}
	}

	Box::new(node)
}

fn create_aql_node_from_attribute<S>(
	name: String,
	value: InputValue<S>,
	scalar: &DbScalarType,
) -> impl AQLNode
where
	S: ScalarValue,
{
	match scalar {
		DbScalarType::String => StringFilter::get_aql_filter_node(name, value),
		_ => todo!(),
	}
}
