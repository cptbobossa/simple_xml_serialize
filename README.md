# Simple XML Service
This is a Rust crate for serialization of data to XML. `XMLElement`s can either be built
manually, or the `simple_xml_serialize_macro` crate can be used to generate `From` implementations for structs. 

## Example Usage
```rust
use simple_xml_serialize::XMLElement;

fn main() {
    // build up your XMLElement with individual calls ...
    let mut ele = XMLElement::new("person");
    ele.add_attr("age", 28); // accept any value that implements `ToString`.
    ele.add_empty_attr("father");
    ele.set_text("John Doe");

    // ... or with the builder pattern
    let sub_ele = XMLElement::new("person")
        .attr("age", 4)
        .empty_attr("daughter")
        .text("Jane Doe");

    ele.add_element(sub_ele); // `add_element` accepts values that implement `Into<XMLElement>`

    let expected = r#"<person age="28" father><person age="4" daughter>Jane Doe</person>John Doe</person>"#;
    assert_eq!(expected, ele.to_string());
    println!("{}",  ele.to_string_pretty("\n", "\t")); // specify your preferred newline and indentation for pretty printing

    ele.set_text("John Doe > John Deere"); // illegal characters in text will be substituted e.g. > becomes &gt;
    let expected = r#"<person age="28" father><person age="4" daughter>Jane Doe</person>John Doe &gt; John Deere</person>"#;
    assert_eq!(expected, ele.to_string());

   
    ele.set_text("<![CDATA[John Doe > John Deere]]>"); // illegal characters in CDATA tags are respected
    let expected = r#"<person age="28" father><person age="4" daughter>Jane Doe</person><![CDATA[John Doe > John Deere]]></person>"#;
    assert_eq!(expected, ele.to_string());
}
```


## Using `simple_xml_serialize_macro`
Using this proc_macro crate allows annotating structs with `#[xml_element("...")]` to generate `From` implementations of your struct to `XMLElement`. Individual fields are annotated with `sxs_type_attr`, `sxs_type_empty_attr`, `sxs_type_text`, `sxs_type_element`, and `sxs_type_multi_element`. Any fields not annotated are ignored.
```rust
use simple_xml_serialize::XMLElement;
use simple_xml_serialize_macro::xml_element;

#[xml_element("custom_name_here")]
struct MyPoint {
    // default for attrs is the name of the field
    #[sxs_type_attr] 
    lon: f32,

    // attrs can be renamed
    #[sxs_type_attr(rename="lat")] 
    latitude: f32,

    #[sxs_type_attr]
    active: bool,
    #[sxs_type_empty_attr]
    grid_system: String,

    // nested XMLElements and collections of XMLElements can be renamed
    #[sxs_type_element] 
    name: MyName,
    #[sxs_type_multi_element(rename="id")] 
    names: Vec<MyName>
}

#[xml_element("Identifier")]
struct MyName {
    #[sxs_type_text]
    val: String,
}

fn main() {
    let my_point = MyPoint {
        latitude: 43.38,
        lon: 60.11,
        active: true,
        grid_system: "wgs84".to_string(),
        name: MyName{val: "p0".to_string()},
        names: vec![MyName{val: "p1".to_string()},MyName{val: "p2".to_string()}]
    };
    
    let my_point_xml = XMLElement::from(my_point); // can also take refs `&my_point`
    let expected = r#"<custom_name_here lon="60.11" lat="43.38" active="true" wgs84><Identifier>p0</Identifier><id>p1</id><id>p2</id></custom_name_here>"#;
    assert_eq!(expected, my_point_xml.to_string());

    let expected = r#"<custom_name_here lon="60.11" lat="43.38" active="true" wgs84>
  <Identifier>
    p0
  </Identifier>
  <id>
    p1
  </id>
  <id>
    p2
  </id>
</custom_name_here>"#;
    assert_eq!(expected, my_point_xml.to_string_pretty("\n", "  ")); 
}
```