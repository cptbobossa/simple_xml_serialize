# simple_xml_serialize_macro
Using this proc_macro crate allows annotating structs with `#[xml_element("...")]` to generate `From` implementations of your struct to `XMLElement`. Individual fields are annotated with `sxs_type_attr`, `sxs_type_empty_attr`, `sxs_type_text`, `sxs_type_element`, and `sxs_type_multi_element`. Any fields not annotated are ignored.

```rust
extern crate simple_xml_serialize;
extern crate simple_xml_serialize_macro;
use simple_xml_serialize::XMLElement;
use simple_xml_serialize_macro::xml_element;


#[xml_element("custom_name_here")]
struct MyPoint {
    #[sxs_type_attr(rename="lat")]
    latitude: f32,
    #[sxs_type_attr]
    lon: f32,
    #[sxs_type_attr]
    active: bool,
    #[sxs_type_element(rename="Identifier")]
    name: MyName,
}

#[xml_element("Name")]
struct MyName {
    #[sxs_type_text]
    val: String,
}

fn main() {
    let my_point = MyPoint {
        latitude: 43.38,
        lon: 60.11,
        active: true,
        name: MyName{val: "p1".to_string()},
    };
    let my_point_xml = XMLElement::from(my_point); // can also take refs `&my_point`
    let expected = r#"<custom_name_here lat="43.38" lon="60.11" active="true"><Identifier>p1</Identifier></custom_name_here>"#;
    assert_eq!(expected, my_point_xml.to_string());

    let expected = r#"<custom_name_here lat="43.38" lon="60.11" active="true">
  <Identifier>
    p1
  </Identifier>
</custom_name_here>"#;
    assert_eq!(expected, my_point_xml.to_string_pretty("\n","  "));
}
```
## Features
There is also a feature `process_options` to allow all the same code to work behind `Option` types. This is behind
a feature gate since generating the code is a bit tricky and I suspect it may be too easy to break. Enable it by adding
`features = ["process_options"]` in your `Cargo.toml`.

```rust,ignore
use simple_xml_serialize::XMLElement;
use simple_xml_serialize_macro::xml_element;

#[xml_element("Employee")]
struct Person1 {
    #[sxs_type_attr(rename="Name")]
    name: String,
    #[sxs_type_attr]
    age: Option<u8>,
}

let person1 = Person1{name: "Robert".to_string(), age: None};
let expected = r#"<Employee Name="Robert"/>"#;
assert_eq!(XMLElement::from(&person1).to_string(), expected);


let person1 = Person1{name: "Robert".to_string(), age: Some(52)};
let expected = r#"<Employee Name="Robert" age="52"/>"#;
assert_eq!(XMLElement::from(&person1).to_string(), expected);
```