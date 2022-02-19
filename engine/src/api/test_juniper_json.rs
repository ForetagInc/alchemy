use crate::juniper::json::{Object, SchemaInfo};
use juniper::graphql_value;
use juniper::{execute_sync, EmptyMutation, EmptySubscription, RootNode, Variables};

#[test]
fn test_sdl_type_info() {
	let sdl = r#"
        type Bar {
            location: String
            capacity: Int
            open: Boolean!
            rating: Float
            foo: Foo
        }
        type Foo {
            message: String
            bar: Bar
        }
        "#;

    let data = serde_json::from_str(
        r#"
        {
            "message": "hello world",
            "bar": {
                "location": "downtown",
                "capacity": 80,
                "open": true,
                "rating": 4.5,
                "foo": {
                    "message": "drink more"
                }
            }
        }"#,
    )
    .unwrap();

    let info = SchemaInfo {
        name: "Foo".to_string(),
        schema: sdl.to_string(),
    };
    let object = Object { fields: data };

    let schema: RootNode<_, _, _> = RootNode::new_with_info(
        object,
        EmptyMutation::new(),
        EmptySubscription::new(),
        info,
        (),
        (),
    );

    // print!("{}", schema.as_schema_language());

    let query = r#"
        {
            message
            bar {
                location
                capacity
                open
                rating
                foo {
                    message
                }
            }
        }"#;

    assert_eq!(
        execute_sync(query, None, &schema, &Variables::new(), &()),
        Ok((
            graphql_value!({
                "message": "hello world",
                "bar": {
                    "location": "downtown",
                    "capacity": 80,
                    "open": true,
                    "rating": 4.5,
                    "foo": {
                        "message": "drink more"
                    }
                }
            }),
            vec![]
        ))
    );
}

#[test]
fn test_required_field() {
    let sdl = r#"
        type Bar {
            location: String
            open: Boolean!
        }
        type Foo {
            message: String
            bar: Bar
        }
        "#;

    let data = serde_json::from_str(
        r#"
        {
            "message": "hello world",
            "bar": {
                "capacity": 80
            }
        }"#,
    )
    .unwrap();

    let info = SchemaInfo {
        name: "Foo".to_string(),
        schema: sdl.to_string(),
    };
    let object = Object { fields: data };

    let schema: RootNode<_, _, _> = RootNode::new_with_info(
        object,
        EmptyMutation::new(),
        EmptySubscription::new(),
        info,
        (),
        (),
    );

    let query = r#"
        {
            message
            bar {
                location
                open
            }
        }"#;

    assert_eq!(
        execute_sync(query, None, &schema, &Variables::new(), &()),
        Ok((
            graphql_value!({
                "message": "hello world",
                "bar": None,
            }),
            vec![]
        ))
    );
}
