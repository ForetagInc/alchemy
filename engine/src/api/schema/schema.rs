use graphql_parser::parse_schema;
use graphql_parser::query::{Text, Type};
use graphql_parser::schema::{Definition, TypeDefinition};
use juniper::meta::{Field, MetaType};
use juniper::{
    Arguments, ExecutionResult, Executor, GraphQLType, GraphQLValue, Registry, ScalarValue, Value,
};
use serde_json::{Map, Value as JsonValue};

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaInfo {
    pub schema: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub fields: Map<String, JsonValue>,
}

impl<S> GraphQLType<S> for Object
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
        let mut fields = Vec::new();

        let ast = parse_schema::<&str>(info.schema.as_str()).unwrap();
        for d in &ast.definitions {
            match &d {
                Definition::TypeDefinition(d) => match d {
                    TypeDefinition::Object(d) => {
                        if d.name == info.name {
                            for field in &d.fields {
                                fields.push(build_field(
                                    info,
                                    registry,
                                    field.name,
                                    field.field_type.clone(),
                                    true,
                                ));
                            }
                        }
                    }
                    _ => todo!(),
                },
                _ => {}
            }
        }
        registry
            .build_object_type::<Object>(info, &fields)
            .into_meta()
    }
}

fn build_field<'r, 't, S, T>(
    info: &SchemaInfo,
    registry: &mut Registry<'r, S>,
    field_name: &str,
    type_ref: Type<'t, T>,
    nullable: bool,
) -> Field<'r, S>
where
    S: 'r + ScalarValue,
    T: Text<'t>,
{
    match type_ref {
        Type::ListType(_nested_type) => {
            todo!()
        }
        Type::NonNullType(nested_type) => {
            build_field(info, registry, field_name, *nested_type, false)
        }
        Type::NamedType(type_name) => match type_name.as_ref() {
            "String" => {
                if nullable {
                    registry.field::<Option<String>>(field_name, &())
                } else {
                    registry.field::<String>(field_name, &())
                }
            }
            "Int" => {
                if nullable {
                    registry.field::<Option<i32>>(field_name, &())
                } else {
                    registry.field::<i32>(field_name, &())
                }
            }
            "Float" => {
                if nullable {
                    registry.field::<Option<f64>>(field_name, &())
                } else {
                    registry.field::<f64>(field_name, &())
                }
            }
            "Boolean" => {
                if nullable {
                    registry.field::<Option<bool>>(field_name, &())
                } else {
                    registry.field::<bool>(field_name, &())
                }
            }
            _ => {
                let field_node_type_info = &SchemaInfo {
                    schema: info.schema.clone(),
                    name: type_name.clone().as_ref().to_string(),
                };
                if nullable {
                    registry.field::<Option<Object>>(field_name, field_node_type_info)
                } else {
                    registry.field::<Object>(field_name, field_node_type_info)
                }
            }
        },
    }
}

impl<S> GraphQLValue<S> for Object
where
    S: ScalarValue,
{
    type Context = ();
    type TypeInfo = SchemaInfo;

    fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
        <Self as GraphQLType<S>>::name(info)
    }

    fn resolve_field(
        &self,
        info: &Self::TypeInfo,
        field_name: &str,
        _args: &Arguments<S>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        let field_value = self.fields.get(field_name);
        match field_value {
            None => Ok(Value::null()),
            Some(field_value) => match field_value {
                JsonValue::Null => Ok(Value::null()),
                JsonValue::Bool(field_value) => Ok(Value::from(field_value.clone())),
                JsonValue::Number(field_value) => {
                    if field_value.is_f64() {
                        Ok(Value::from(field_value.as_f64().unwrap()))
                    } else if field_value.is_i64() {
                        Ok(Value::from(field_value.as_i64().unwrap() as i32))
                    } else if field_value.is_u64() {
                        Ok(Value::from(field_value.as_u64().unwrap() as i32))
                    } else {
                        panic!("unexpected case")
                    }
                }
                JsonValue::String(field_value) => Ok(Value::from(field_value.clone())),
                JsonValue::Array(_field_value) => {
                    todo!()
                }
                JsonValue::Object(field_value) => {
                    let meta_type = executor
                        .schema()
                        .concrete_type_by_name(<Self as GraphQLType<S>>::name(info).unwrap())
                        .expect("Type not found in schema");
                    let field_type = &meta_type
                        .field_by_name(field_name)
                        .unwrap()
                        .field_type
                        .innermost_name();
                    let filed_type_name = field_type;

                    executor.resolve::<Object>(
                        &SchemaInfo {
                            schema: "".to_string(),
                            name: filed_type_name.to_string(),
                        },
                        &Object {
                            fields: field_value.clone(),
                        },
                    )
                }
            },
        }
    }
}
