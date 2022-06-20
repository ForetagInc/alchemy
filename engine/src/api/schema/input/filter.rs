use convert_case::Casing;
use juniper::meta::{Argument, MetaType};
use juniper::{
	Arguments, FromInputValue, GraphQLType, GraphQLValue, InputValue, Registry, ScalarValue, ID,
};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::api::schema::input::string_filter::{StringFilter, StringFilterData};
use crate::api::schema::operations::OperationData;
use crate::lib::database::api::DbScalarType;
use crate::lib::database::aql::{AQLFilter, AQLLogicalFilter, AQLLogicalOperator, AQLNode};

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

impl<'a, S> EntityFilterData<'a, S>
where
	S: ScalarValue,
{
	pub fn new(data: &'a OperationData<S>) -> Self {
		Self {
			name: format!(
				"{}BoolExp",
				data.entity.name.to_case(convert_case::Case::Pascal)
			),
			operation_data: data,
		}
	}
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
	let mut and = None;
	let mut not = Box::new(None);
	let mut or = None;

	match data {
		InputValue::Object(items) => {
			for (key, value) in items {
				match key.item.as_str() {
					"_and" => and = Some(collect_filter_attributes(&value.item)),
					"_not" => not = Box::new(Some(parse_filter_attributes(&value.item))),
					"_or" => or = Some(collect_filter_attributes(&value.item)),
					&_ => {
						attributes.insert(key.item.clone(), value.item.clone());
					}
				}
			}
		}
		_ => {}
	}

	FilterAttributes {
		attributes,
		and,
		not,
		or,
	}
}

fn collect_filter_attributes<S>(data: &InputValue<S>) -> Vec<FilterAttributes<S>>
where
	S: ScalarValue,
{
	let mut arr = Vec::new();

	if let InputValue::List(elements) = data {
		for el in elements {
			arr.push(parse_filter_attributes(&el.item))
		}
	} else if let InputValue::Object(_) = data {
		arr.push(parse_filter_attributes(&data))
	}

	arr
}

pub fn get_aql_filter_from_args<S>(
	args: &Arguments<S>,
	data: &OperationData<S>,
) -> Option<Box<dyn AQLNode>>
where
	S: ScalarValue,
{
	if let Some(entity_filter) = args.get::<EntityFilter<S>>("where") {
		let properties: HashMap<String, DbScalarType> = data
			.entity
			.properties
			.iter()
			.map(|p| (p.name.clone(), p.scalar_type.clone()))
			.collect();

		get_aql_filter_from_entity_filter(&entity_filter.filter_arguments, &properties)
	} else {
		None
	}
}

pub fn get_aql_filter_from_entity_filter<S>(
	filter: &FilterAttributes<S>,
	properties: &HashMap<String, DbScalarType>,
) -> Option<Box<dyn AQLNode>>
where
	S: ScalarValue,
{
	let attr_node = Box::new(create_aql_node_from_attributes(&filter, &properties));

	let mut and_node = None;
	let mut or_node = None;
	let mut not_node = None;

	fn collect_logical_node<S: ScalarValue>(
		filters: &Vec<FilterAttributes<S>>,
		operation: AQLLogicalOperator,
		properties: &HashMap<String, DbScalarType>,
	) -> Option<Box<dyn AQLNode>> {
		let mut n = AQLLogicalFilter {
			nodes: Vec::new(),
			operation,
		};

		for a in filters {
			if let Some(f) = get_aql_filter_from_entity_filter(a, properties) {
				n.nodes.push(f);
			}
		}

		if n.nodes.len() > 0 {
			Some(Box::new(n))
		} else {
			None
		}
	}

	if let Some(and) = &filter.and {
		and_node = collect_logical_node(and, AQLLogicalOperator::AND, properties);
	}

	if let Some(not) = &*filter.not {
		not_node = get_aql_filter_from_entity_filter(&not, properties);
	}

	if let Some(or) = &filter.or {
		or_node = collect_logical_node(or, AQLLogicalOperator::OR, properties);
	}

	let node = AQLFilter {
		attr_node,
		and_node,
		or_node,
		not_node,
	};

	if node.valid() {
		Some(Box::new(node))
	} else {
		None
	}
}

fn create_aql_node_from_attributes<S>(
	filter: &FilterAttributes<S>,
	properties: &HashMap<String, DbScalarType>,
) -> impl AQLNode
where
	S: ScalarValue,
{
	let mut node = AQLLogicalFilter {
		nodes: Vec::new(),
		operation: AQLLogicalOperator::AND,
	};

	for (name, value) in &filter.attributes {
		if let Some(scalar) = properties.get(name) {
			node.nodes.push(Box::new(create_aql_node_from_attribute(
				name.to_string(),
				value,
				scalar,
			)));
		}
	}

	node
}

fn create_aql_node_from_attribute<S>(
	name: String,
	value: &InputValue<S>,
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

pub struct EntityIndicesFilterData<'a, S>
where
	S: ScalarValue,
{
	pub name: String,
	pub operation_data: &'a OperationData<S>,
}

impl<'a, S> EntityIndicesFilterData<'a, S>
where
	S: ScalarValue,
{
	pub fn new(data: &'a OperationData<S>) -> Self {
		Self {
			name: format!(
				"{}IndexFilter",
				data.entity.name.to_case(convert_case::Case::Pascal)
			),
			operation_data: data,
		}
	}
}

#[derive(Debug)]
pub struct EntityIndicesFilter<'a, S>
where
	S: ScalarValue + 'a,
{
	pub indices_arguments: HashMap<String, InputValue<S>>,

	_marker: PhantomData<&'a S>,
}

impl<'a, S> GraphQLValue<S> for EntityIndicesFilter<'a, S>
where
	S: ScalarValue,
{
	type Context = ();
	type TypeInfo = EntityIndicesFilterData<'a, S>;

	fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
		<Self as GraphQLType<S>>::name(info)
	}
}

impl<'a, S> GraphQLType<S> for EntityIndicesFilter<'a, S>
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
		// TODO: add real indices from database
		let key = registry.arg::<ID>("_key", &());

		registry
			.build_input_object_type::<Self>(info, &vec![key])
			.into_meta()
	}
}

impl<'a, S> FromInputValue<S> for EntityIndicesFilter<'a, S>
where
	S: ScalarValue,
{
	fn from_input_value(value: &InputValue<S>) -> Option<Self> {
		Some(Self {
			indices_arguments: parse_indices_attributes(value),

			_marker: Default::default(),
		})
	}
}

pub fn parse_indices_attributes<S>(data: &InputValue<S>) -> HashMap<String, InputValue<S>>
where
	S: ScalarValue,
{
	let mut attributes = HashMap::new();

	match data {
		InputValue::Object(items) => {
			for (key, value) in items {
				attributes.insert(key.item.clone(), value.item.clone());
			}
		}
		_ => unreachable!(),
	}

	attributes
}
