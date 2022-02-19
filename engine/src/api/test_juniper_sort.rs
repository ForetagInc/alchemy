use graphql_parser::parse_schema;
use crate::api::schema::sort;

#[test]
fn test_sort() {

    let sdl = r#"
type Foo {
  message: String
  bar: Bar
}
type Bar {
  location: String
  open: Boolean!
}
"#;

    let mut ast = parse_schema::<&str>(sdl).expect("valid graphql schema");

    let unsortted = ast.format(&Default::default());
    assert_eq!(unsortted.trim(), sdl.trim());

    sort::document(&mut ast);

    let sortted = ast.format(&Default::default());
    assert_eq!(sortted.trim(), r#"
type Bar {
  location: String
  open: Boolean!
}
type Foo {
  bar: Bar
  message: String
}
"#.trim());

}