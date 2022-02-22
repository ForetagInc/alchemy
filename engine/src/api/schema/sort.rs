use graphql_parser::schema::*;

pub fn document<'a, T>(value: &mut Document<'a, T>)
where
    T: Text<'a>,
{
    for definition in value.definitions.iter_mut() {
        match definition {
            Definition::SchemaDefinition(x) => schema_definition(x),
            Definition::TypeDefinition(x) => type_definition(x),
            Definition::TypeExtension(x) => type_extension(x),
            Definition::DirectiveDefinition(x) => directive_definition(x),
        }
    }
    value.definitions.sort_by_key(|f| match f {
        Definition::SchemaDefinition(_) => "1:".to_string(),
        Definition::DirectiveDefinition(x) => format!("2:{}", x.name.as_ref()),
        Definition::TypeDefinition(x) => format!(
            "3:{}",
            match x {
                TypeDefinition::Scalar(x) => format!("1:{}", x.name.as_ref()),
                TypeDefinition::Enum(x) => format!("2:{}", x.name.as_ref()),
                TypeDefinition::InputObject(x) => format!("3:{}", x.name.as_ref()),
                TypeDefinition::Interface(x) => format!("4:{}", x.name.as_ref()),
                TypeDefinition::Object(x) => format!("5:{}", x.name.as_ref()),
                TypeDefinition::Union(x) => format!("6:{}", x.name.as_ref()),
            }
        ),
        Definition::TypeExtension(x) => format!(
            "4:{}",
            match x {
                TypeExtension::Scalar(x) => format!("1:{}", x.name.as_ref()),
                TypeExtension::Enum(x) => format!("2:{}", x.name.as_ref()),
                TypeExtension::InputObject(x) => format!("3:{}", x.name.as_ref()),
                TypeExtension::Interface(x) => format!("4:{}", x.name.as_ref()),
                TypeExtension::Object(x) => format!("5:{}", x.name.as_ref()),
                TypeExtension::Union(x) => format!("6:{}", x.name.as_ref()),
            }
        ),
    });
}

fn schema_definition<'a, T>(value: &mut SchemaDefinition<'a, T>)
where
    T: Text<'a>,
{
    for directive in value.directives.iter_mut() {
        directive.arguments.sort_by_key(|(x, _)| x.to_owned())
    }
    value.directives.sort_by_key(|d| d.name.clone())
}

fn type_definition<'a, T>(value: &mut TypeDefinition<'a, T>)
where
    T: Text<'a>,
{
    match value {
        TypeDefinition::Scalar(_) => {}
        TypeDefinition::Object(x) => object_type(x),
        TypeDefinition::Interface(x) => interface_type(x),
        TypeDefinition::Union(x) => union_type(x),
        TypeDefinition::Enum(x) => enum_type(x),
        TypeDefinition::InputObject(x) => input_object_type(x),
    }
}

fn type_extension<'a, T>(value: &mut TypeExtension<'a, T>)
where
    T: Text<'a>,
{
    match value {
        TypeExtension::Scalar(_) => {}
        TypeExtension::Object(x) => object_type_extension(x),
        TypeExtension::Interface(x) => interface_type_extension(x),
        TypeExtension::Union(x) => union_type_extension(x),
        TypeExtension::Enum(x) => enum_type_extension(x),
        TypeExtension::InputObject(x) => input_object_type_extension(x),
    }
}

fn directive_definition<'a, T>(value: &mut DirectiveDefinition<'a, T>)
where
    T: Text<'a>,
{
    value.arguments.sort_by_key(|x| x.name.clone());
    value.locations.sort_by_key(|l| match l {
        DirectiveLocation::Query => 1,
        DirectiveLocation::Mutation => 2,
        DirectiveLocation::Subscription => 3,
        DirectiveLocation::Field => 4,
        DirectiveLocation::FragmentDefinition => 5,
        DirectiveLocation::FragmentSpread => 6,
        DirectiveLocation::InlineFragment => 7,
        DirectiveLocation::Schema => 8,
        DirectiveLocation::Scalar => 9,
        DirectiveLocation::Object => 10,
        DirectiveLocation::FieldDefinition => 11,
        DirectiveLocation::ArgumentDefinition => 12,
        DirectiveLocation::Interface => 13,
        DirectiveLocation::Union => 14,
        DirectiveLocation::Enum => 15,
        DirectiveLocation::EnumValue => 16,
        DirectiveLocation::InputObject => 17,
        DirectiveLocation::InputFieldDefinition => 18,
    });
}

fn object_type<'a, T>(value: &mut ObjectType<'a, T>)
where
    T: Text<'a>,
{
    value.implements_interfaces.sort();
    fields(&mut value.fields);
}

fn object_type_extension<'a, T>(value: &mut ObjectTypeExtension<'a, T>)
where
    T: Text<'a>,
{
    value.implements_interfaces.sort();
    fields(&mut value.fields);
}

fn fields<'a, T>(value: &mut Vec<Field<'a, T>>)
where
    T: Text<'a>,
{
    for f in value.iter_mut() {
        f.arguments.sort_by_key(|x| x.name.clone())
    }
    value.sort_by_key(|x| x.name.clone());
}

fn interface_type<'a, T>(value: &mut InterfaceType<'a, T>)
where
    T: Text<'a>,
{
    fields(&mut value.fields);
}

fn interface_type_extension<'a, T>(value: &mut InterfaceTypeExtension<'a, T>)
where
    T: Text<'a>,
{
    fields(&mut value.fields);
}

fn union_type<'a, T>(value: &mut UnionType<'a, T>)
where
    T: Text<'a>,
{
    value.types.sort()
}

fn union_type_extension<'a, T>(value: &mut UnionTypeExtension<'a, T>)
where
    T: Text<'a>,
{
    value.types.sort()
}

fn enum_type<'a, T>(value: &mut EnumType<'a, T>)
where
    T: Text<'a>,
{
    value.values.sort_by_key(|x| x.name.clone())
}

fn enum_type_extension<'a, T>(value: &mut EnumTypeExtension<'a, T>)
where
    T: Text<'a>,
{
    value.values.sort_by_key(|x| x.name.clone())
}

fn input_object_type<'a, T>(value: &mut InputObjectType<'a, T>)
where
    T: Text<'a>,
{
    value.fields.sort_by_key(|x| x.name.clone())
}

fn input_object_type_extension<'a, T>(value: &mut InputObjectTypeExtension<'a, T>)
where
    T: Text<'a>,
{
    value.fields.sort_by_key(|x| x.name.clone())
}
