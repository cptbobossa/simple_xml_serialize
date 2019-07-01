/*!
# Simple XML Serialization
This is a Rust crate for serialization of data to XML. `XMLElement`s can either be built
manually, or the `simple_xml_serialize_macro` crate can be used to generate `From` implementations for structs. 

## Example Usage
```rust
use simple_xml_serialize::XMLElement;

fn main() {
    // build up your XMLElement with individual calls ...
    let mut ele = XMLElement::new("person");
    ele.add_attr("age", 28); // accept any value that implements `ToString`.
    ele.set_text("John Doe");

    // ... or with the builder pattern
    let sub_ele = XMLElement::new("person")
        .attr("age", 4)
        .text("Jane Doe");

    ele.add_element(sub_ele); // `add_element` accepts values that implement `Into<XMLElement>`

    let expected = r#"<person age="28"><person age="4">Jane Doe</person>John Doe</person>"#;
    assert_eq!(expected, ele.to_string());
    println!("{}",  ele.to_string_pretty("\n", "\t")); // specify your preferred newline and indentation for pretty printing

    ele.set_text("John Doe > John Deere"); // illegal characters in text will be substituted e.g. > becomes &gt;
    let expected = r#"<person age="28"><person age="4">Jane Doe</person>John Doe &gt; John Deere</person>"#;
    assert_eq!(expected, ele.to_string());

   
    ele.set_text("<![CDATA[John Doe > John Deere]]>"); // illegal characters in CDATA tags are respected
    let expected = r#"<person age="28"><person age="4">Jane Doe</person><![CDATA[John Doe > John Deere]]></person>"#;
    assert_eq!(expected, ele.to_string());
}
```
*/

use std::fmt;

/// The basic type this crate provides. Functions are provided for setting/adding to the fields in this struct.
/// Any manipulation past that is left to the user by accessing the fields directly.
#[derive(Clone,PartialEq,Debug)]
pub struct XMLElement {
    /// The tag for this element node. IE `<myelement/>`
    pub name: String,
    /// Nested XMLElements. IE `<myelement><nested/></myelement>`
    pub contents: Option<Vec<XMLElement>>,
    /// Plain character data inside of the node. IE `<myelement>hello world</myelement>`
    pub text: Option<String>,
    /// The key/value pairs inside of an element tag. IE `<myelement attr1="hello" attr2="world"/>`
    pub attrs: Option<Vec<XMLAttr>>,
}

impl fmt::Display for XMLElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = String::new();
        ret.push('<');
        ret.push_str(&self.name);
        
        if let Some(ref attrs) = self.attrs {
            for a in attrs {
                ret.push(' ');
                ret.push_str(&a.name);
                ret.push('=');
                ret.push('"');
                ret.push_str(&a.value);
                ret.push('"');
            }
        }
        if self.contents.is_none() && self.text.is_none() {
            ret.push('/');
            ret.push('>');
        } else {
            ret.push('>');

            if let Some(contents) = &self.contents {
                for c in contents {
                    ret.push_str(&c.to_string());
                }
            }
            if let Some(text) = &self.text {
                let (before_cdata, opt_cdata) = split_cdata(&text);
                let text = before_cdata.replace("&", "&amp;");
                let text = text.replace("<", "&lt;");
                let text = text.replace(">", "&gt;");
                let text = text.replace("'", "&apos;");
                let text = text.replace(r#"""#, "&quot;");
                ret.push_str(&text);
                if let Some((cdata, after_cdata)) = opt_cdata {
                    ret.push_str(&cdata);
                    let text = after_cdata.replace("&", "&amp;");
                    let text = text.replace("<", "&lt;");
                    let text = text.replace(">", "&gt;");
                    let text = text.replace("'", "&apos;");
                    let text = text.replace(r#"""#, "&quot;");
                    ret.push_str(&text);
                }
            }

            ret.push_str(&format!("</{}>", self.name));
        }
        write!(f, "{}", ret)
    }
}


fn split_cdata(text: &str) -> (String, Option<(String, String)>) {
    let cdata_start = "<![CDATA[";
    let cdata_end = "]]>";
    let csi = match text.find(&cdata_start) {
        None => {return (text.to_string(), None)},
        Some(index) => index,
    };
    let cei = match text[csi..].find(&cdata_end) {
        None => {return (text.to_string(), None)},
        Some(index) => csi+index+3,
    };
    let before_cdata = String::from(&text[..csi]);
    let cdata_section = String::from(&text[csi..cei]);
    let after_cdata = String::from(&text[cei..]);
    return (before_cdata, Some((cdata_section, after_cdata)));
}

impl From<&XMLElement> for XMLElement {
    fn from(e: &XMLElement) -> Self {
        e.clone()
    }
}

impl XMLElement {

    /// Constructs a new XMLElement with the given name and `None` for the rest of the fields
    /// # Arguments
    /// 
    /// * `name` - A string slice that holds the name of the XML element displayed at the element root
    ///
    /// # Example
    ///
    /// ```
    /// use simple_xml_serialize::XMLElement;
    /// let ele = XMLElement::new("name");
    /// assert_eq!(ele.to_string(), String::from("<name/>"));
    /// ```
    pub fn new(name: &str) -> Self {
        XMLElement{
            name: String::from(name),
            contents: None,
            attrs: None,
            text: None,
        }
    }

    /// Builder pattern function for changing the name of an XMLElement
    /// # Arguments
    /// 
    /// * `name` - A string slice that holds the name of the XML element displayed at the element root
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// let ele = XMLElement::new("name").name("changed");
    /// assert_eq!(ele.to_string(), String::from("<changed/>"));
    /// ```
    pub fn name(mut self, name: &str) -> Self {
        self.set_name(name);
        self
    }

    /// Changes the name of an XMLElement
    /// # Arguments
    /// 
    /// * `name` - A string slice that holds the name of the XML element displayed at the element root
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// let mut ele = XMLElement::new("name");
    /// ele.set_name("changed");
    /// assert_eq!(ele.to_string(), String::from("<changed/>"));
    /// ```
    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }

    /// Builder pattern function for adding an attribute to the XMLElement
    /// # Arguments
    /// 
    /// * `attr` - A string slice that holds the name of the attribute
    /// * `attr_val` - Any type that implements ToString; the value of the attribute
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// let ele = XMLElement::new("name").attr("my_attr", 1);
    /// assert_eq!(ele.to_string(), String::from(r#"<name my_attr="1"/>"#));
    /// ```
    pub fn attr(mut self, attr: &str, attr_val: impl ToString) -> Self {
        self.add_attr(attr, attr_val);
        self
    }

    /// Adds an attribute to the XMLElement
    /// # Arguments
    /// 
    /// * `attr` - A string slice that holds the name of the attribute
    /// * `attr_val` - Any type that implements ToString; the value of the attribute
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// let mut ele = XMLElement::new("name");
    /// ele.add_attr("my_attr", 1);
    /// assert_eq!(ele.to_string(), String::from(r#"<name my_attr="1"/>"#));
    /// ```
    pub fn add_attr(&mut self, attr: &str, attr_val: impl ToString) {
        if let Some(ref mut attr_vec) = self.attrs {
            let new_attr: XMLAttr = XMLAttr{
                name: String::from(attr),
                value: attr_val.to_string(),
            };
            attr_vec.push(new_attr);
        } else {
            let mut attr_vec: Vec<XMLAttr> = Vec::new();
            let new_attr: XMLAttr = XMLAttr{
                name: String::from(attr),
                value: attr_val.to_string(),
            };
            attr_vec.push(new_attr);
            self.attrs = Some(attr_vec);
        }
    }

    /// Builder pattern function for adding an element to the contents of this XMLElement
    /// # Arguments
    /// 
    /// * `new_ele` - Any type that implements `Into<XMLElement>`
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// struct MyPoint {}
    /// impl From<MyPoint> for XMLElement {
    ///     fn from(p: MyPoint) -> Self {
    ///         XMLElement::new("point")
    ///     }
    /// }
    /// let ele = XMLElement::new("name").element(MyPoint{});
    /// assert_eq!(ele.to_string(), String::from("<name><point/></name>"));
    /// ```
    pub fn element(mut self, new_ele: impl Into<XMLElement>) -> Self {
        self.add_element(new_ele.into());
        self
    }

    /// Adds an element to the contents of this XMLElement
    /// # Arguments
    /// 
    /// * `new_ele` - Any type that implements `Into<XMLElement>`
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// # struct MyPoint {}
    /// # impl From<MyPoint> for XMLElement {
    /// #     fn from(p: MyPoint) -> Self {
    /// #         XMLElement::new("point")
    /// #     }
    /// # }
    /// let mut ele = XMLElement::new("name");
    /// ele.add_element(MyPoint{});
    /// assert_eq!(ele.to_string(), String::from("<name><point/></name>"));
    /// ```
    pub fn add_element(&mut self, new_ele: impl Into<XMLElement>) {
        if let Some(ref mut ele_vec) = self.contents {
            ele_vec.push(new_ele.into());
        } else {
            let mut ele_vec: Vec<XMLElement> = Vec::new();
            ele_vec.push(new_ele.into());
            self.contents = Some(ele_vec);
        }
    }

    /// Builder pattern for adding a collection of elements to the contents of this XMLElement
    /// # Arguments
    /// 
    /// * `new_eles` - Any collection of `Into<XMLElement>` as long as that collection implements `IntoIterator`
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// # struct MyPoint {}
    /// # impl From<MyPoint> for XMLElement {
    /// #     fn from(p: MyPoint) -> Self {
    /// #         XMLElement::new("point")
    /// #     }
    /// # }
    /// let points: Vec<MyPoint> = vec![MyPoint{}, MyPoint{}];
    /// let mut ele = XMLElement::new("name").elements(points);
    /// assert_eq!(ele.to_string(), String::from("<name><point/><point/></name>"));
    /// ```
    pub fn elements<T>(mut self, new_eles: T) -> Self 
        where T: IntoIterator, T::Item : Into<XMLElement>,
    {
        self.add_elements(new_eles);
        self
    }

    /// Adds a collection of elements to the contents of this XMLElement
    /// # Arguments
    /// 
    /// * `new_eles` - Any collection of `Into<XMLElement>` as long as that collection implements `IntoIterator`
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// # struct MyPoint {}
    /// # impl From<MyPoint> for XMLElement {
    /// #     fn from(p: MyPoint) -> Self {
    /// #         XMLElement::new("point")
    /// #     }
    /// # }
    /// let points: Vec<MyPoint> = vec![MyPoint{}, MyPoint{}];
    /// let mut ele = XMLElement::new("name");
    /// ele.add_elements(points);
    /// assert_eq!(ele.to_string(), String::from("<name><point/><point/></name>"));
    /// ```
    pub fn add_elements<T>(&mut self, new_eles: T) 
        where T: IntoIterator, T::Item : Into<XMLElement>,
    {
        for ele in new_eles.into_iter() {
            self.add_element(ele);
        }
    }

    /// Adds a collection of elements to the contents of this XMLElement and changes the element name of each
    /// # Arguments
    /// 
    /// * `new_name` - A string slice containing the name to use when adding the elements
    /// * `new_eles` - Any collection of `Into<XMLElement>` as long as that collection implements `IntoIterator`
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// # struct MyPoint {}
    /// # impl From<MyPoint> for XMLElement {
    /// #     fn from(p: MyPoint) -> Self {
    /// #         XMLElement::new("point")
    /// #     }
    /// # }
    /// let mut ele = XMLElement::new("name");
    /// let points: Vec<MyPoint> = vec![MyPoint{}, MyPoint{}];
    /// ele.add_elements_with_name("p", points);
    /// assert_eq!(ele.to_string(), String::from("<name><p/><p/></name>"));
    /// ```
    pub fn add_elements_with_name<T>(&mut self, new_name: &str, new_eles: T) 
        where T: IntoIterator, T::Item : Into<XMLElement>,
    {
        for ele in new_eles.into_iter() {
            let new_element: XMLElement = ele.into();
            self.add_element(new_element.name(new_name));
        }
    }

    /// Builder pattern function for adding raw text to the contents of the XMLElement.
    /// In the ToString implementation fo XMLElement, raw text is always placed after all other contents.
    /// # Arguments
    /// 
    /// * `text` - Any type that implements ToString; text in the element
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// let ele = XMLElement::new("name").text("Some content");
    /// assert_eq!(ele.to_string(), String::from("<name>Some content</name>"));
    /// ```
    pub fn text(mut self, text: impl ToString) -> Self {
        self.set_text(text);
        self
    }

    /// Adds raw text to the contents of the XMLElement. In the ToString implementation for
    /// XMLElement, raw text is always placed after all other contents.
    /// # Arguments
    /// 
    /// * `text` - Any type that implements ToString; text in the element
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// let mut ele = XMLElement::new("name");
    /// ele.set_text("Some content");
    /// assert_eq!(ele.to_string(), String::from("<name>Some content</name>"));
    /// ```
    pub fn set_text(&mut self, text: impl ToString) {
        self.text = Some(text.to_string());
    }

    /// Returns the string representation of the XMLElement, but with newlines and the given indentation
    /// # Arguments
    /// 
    /// * `indent` - A string slice containing the characters to use to indent the document
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// # struct MyPoint {}
    /// # impl From<MyPoint> for XMLElement {
    /// #     fn from(p: MyPoint) -> Self {
    /// #         XMLElement::new("point")
    /// #     }
    /// # }
    /// let mut ele = XMLElement::new("name");
    /// ele.set_text("Some content");
    /// ele.add_element(MyPoint{});
    /// let expected = String::from(r#"<name>
    ///   <point/>
    ///   Some content
    /// </name>"#);
    /// assert_eq!(ele.to_string_pretty("\n", "  "), expected);
    /// ```
    pub fn to_string_pretty(&self, newline: &str, indent: &str) -> String {
        let mut ret = String::new();
        ret.push('<');
        ret.push_str(&self.name);
        
        if let Some(ref attrs) = self.attrs {
            for a in attrs {
                ret.push(' ');
                ret.push_str(&a.name);
                ret.push('=');
                ret.push('"');
                ret.push_str(&a.value);
                ret.push('"');
            }
        }
        if self.contents.is_none() && self.text.is_none() {
            ret.push('/');
            ret.push('>');
        } else {
            ret.push('>');

            let mut intermediate_ret = String::new();

            if let Some(contents) = &self.contents {
                for c in contents {
                    intermediate_ret.push_str(&c.to_string_pretty(newline, indent));
                    intermediate_ret.push_str(newline);
                }
            }
            if let Some(text) = &self.text {
                let (before_cdata, opt_cdata) = split_cdata(&text);
                let text = before_cdata.replace("&", "&amp;");
                let text = text.replace("<", "&lt;");
                let text = text.replace(">", "&gt;");
                let text = text.replace("'", "&apos;");
                let text = text.replace(r#"""#, "&quot;");
                intermediate_ret.push_str(&text);
                if let Some((cdata, after_cdata)) = opt_cdata {
                    intermediate_ret.push_str(&cdata);
                    let text = after_cdata.replace("&", "&amp;");
                    let text = text.replace("<", "&lt;");
                    let text = text.replace(">", "&gt;");
                    let text = text.replace("'", "&apos;");
                    let text = text.replace(r#"""#, "&quot;");
                    intermediate_ret.push_str(&text);
                }
            }
            for l in intermediate_ret.lines() {
                ret.push_str(newline);
                ret.push_str(indent);
                ret.push_str(l);
            }
            ret.push_str(newline);
            ret.push_str(&format!("</{}>", self.name));
        }
        
        ret
    }

    /// Returns the string representation of the XMLElement, but with newlines and the given indentation and an xml prolog
    /// # Arguments
    /// 
    /// * `indent` - A string slice containing the characters to use to indent the document
    ///
    /// # Example
    ///
    /// ```
    /// # use simple_xml_serialize::XMLElement;
    /// # struct MyPoint {}
    /// # impl From<MyPoint> for XMLElement {
    /// #     fn from(p: MyPoint) -> Self {
    /// #         XMLElement::new("point")
    /// #     }
    /// # }
    /// let mut ele = XMLElement::new("name");
    /// ele.set_text("Some content");
    /// ele.add_element(MyPoint{});
    /// let expected = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
    /// <name>
    ///   <point/>
    ///   Some content
    /// </name>"#);
    /// assert_eq!(ele.to_string_pretty_prolog("\n", "  "), expected);
    /// ```
    pub fn to_string_pretty_prolog(&self, newline: &str, indent: &str) -> String {
        let mut ret = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        ret.push_str(newline);
        ret.push_str(&self.to_string_pretty(newline, indent));
        ret
    }
}

/// A key/value pair that is serialized inside the opening tag of an XMLElement.
#[derive(Clone,PartialEq,Debug)]
pub struct XMLAttr {
    pub name: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xmlelement_eq() {
        let ele1 = XMLElement::new("test_element");
        let mut ele2 = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
        ele2.text = Some(String::from("hey"));
        assert_ne!(ele1, ele2);
    }

    #[test]
    fn xmlelement_new() {
        let newele = XMLElement::new("test_element");
        let testele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: None,
            attrs: None,
        };
        assert_eq!(newele, testele);
    }


    #[test]
    fn xmlelement_name() {
        let ele1 = XMLElement::new("test_element").name("new_name");
        let ele2 = XMLElement{
            name: String::from("new_name"),
            contents: None,
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_set_name() {
        let mut ele1 = XMLElement::new("test_element");
        ele1.set_name("new_name");
        let ele2 = XMLElement{
            name: String::from("new_name"),
            contents: None,
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }


    #[test]
    fn xmlelement_attr() {
        let ele1 = XMLElement::new("test_element").attr("a1", 42);
        let test_attr = XMLAttr{name: String::from("a1"), value: 42.to_string()};
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: None,
            attrs: Some(vec![test_attr]),
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_add_attr() {
        let mut ele1 = XMLElement::new("test_element");
        ele1.add_attr("a1", 42);
        let test_attr = XMLAttr{name: String::from("a1"), value: 42.to_string()};
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: None,
            attrs: Some(vec![test_attr]),
        };
        assert_eq!(ele1, ele2);
    }

    struct Point {
        lat: f32,
        lon: f32,
    }

    impl From<Point> for XMLElement {
        fn from(p: Point) -> Self {
            XMLElement::new("point").attr("lat", p.lat).attr("lon", p.lon)
        }
    }



    #[test]
    fn xmlelement_element() {
        let ele1 = XMLElement::new("test_element").element(Point{lat: 12.3, lon: 45.6});
        
        let point_ele: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele]),
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_add_element() {
        let mut ele1 = XMLElement::new("test_element");
        ele1.add_element(Point{lat: 12.3, lon: 45.6});
        
        let point_ele: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele]),
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_elements() {
        let ele_vec = vec![Point{lat: 12.3, lon: 45.6}, Point{lat: 32.1, lon: 65.4}];
        let ele1 = XMLElement::new("test_element").elements(ele_vec);
        
        let point_ele1: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        let point_ele2: XMLElement = Point{lat: 32.1, lon: 65.4}.into();
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele1, point_ele2]),
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_add_elements() {
        let ele_vec = vec![Point{lat: 12.3, lon: 45.6}, Point{lat: 32.1, lon: 65.4}];
        let mut ele1 = XMLElement::new("test_element");
        ele1.add_elements(ele_vec);
        
        let point_ele1: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        let point_ele2: XMLElement = Point{lat: 32.1, lon: 65.4}.into();
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele1, point_ele2]),
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_add_elements_with_name() {
        let ele_vec = vec![Point{lat: 12.3, lon: 45.6}, Point{lat: 32.1, lon: 65.4}];
        let mut ele1 = XMLElement::new("test_element");
        ele1.add_elements_with_name("new_name", ele_vec);
        
        let mut point_ele1: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        point_ele1.set_name("new_name");
        let mut point_ele2: XMLElement = Point{lat: 32.1, lon: 65.4}.into();
        point_ele2.set_name("new_name");
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele1, point_ele2]),
            text: None,
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_text() {
        let ele1 = XMLElement::new("test_element").text("some content");
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("some content")),
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_set_text() {
        let mut ele1 = XMLElement::new("test_element");
        ele1.set_text("some content");
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("some content")),
            attrs: None,
        };
        assert_eq!(ele1, ele2);
    }

    #[test]
    fn xmlelement_to_string() {
        let expected = r#"<test_element a1="42" a2="24"><point lat="12.3" lon="45.6"/><point lat="32.1" lon="65.4"/>some content</test_element>"#;
        

        let point_ele1: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        let point_ele2: XMLElement = Point{lat: 32.1, lon: 65.4}.into();
        let test_attr1 = XMLAttr{name: String::from("a1"), value: 42.to_string()};
        let test_attr2 = XMLAttr{name: String::from("a2"), value: 24.to_string()};
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele1, point_ele2]),
            text: Some(String::from("some content")),
            attrs: Some(vec![test_attr1, test_attr2]),
        };

        assert_eq!(expected, ele2.to_string());
    }

    #[test]
    fn xmlelement_to_string_cdata1() {
        let expected = r#"<test_element><![CDATA[1<2]]></test_element>"#;
        
        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("<![CDATA[1<2]]>")),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string());
    }

    #[test]
    fn xmlelement_to_string_cdata2() {
        let expected = r#"<test_element>1&lt;2<![CDATA[1<2]]>1&lt;2</test_element>"#;
        
        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("1<2<![CDATA[1<2]]>1<2")),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string());
    }

    #[test]
    fn xmlelement_to_string_entity_ref_lt() {
        let expected = r#"<test_element>1&lt;2</test_element>"#;
        
        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("1<2")),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string());
    }

    #[test]
    fn xmlelement_to_string_entity_ref_gt() {
        let expected = r#"<test_element>3&gt;2</test_element>"#;
        
        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("3>2")),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string());
    }
    #[test]
    fn xmlelement_to_string_entity_ref_amp() {
        let expected = r#"<test_element>5&amp;1=1</test_element>"#;
        
        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("5&1=1")),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string());
    }
    #[test]
    fn xmlelement_to_string_entity_ref_apos() {
        let expected = r#"<test_element>&apos;a</test_element>"#;
        
        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("'a")),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string());
    }
    #[test]
    fn xmlelement_to_string_entity_ref_quot() {
        let expected = r#"<test_element>&quot;Hello World&quot;</test_element>"#;
        
        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from(r#""Hello World""#)),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string());
    }

    #[test]
    fn xmlelement_to_string_pretty1() {
        let expected = format!(r#"<test_element a1="42" a2="24">{}<point lat="12.3" lon="45.6"/>{}<point lat="32.1" lon="65.4"/>{}some content{}</test_element>"#, "\n\t", "\n\t", "\n\t", "\n");

        let point_ele1: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        let point_ele2: XMLElement = Point{lat: 32.1, lon: 65.4}.into();
        let test_attr1 = XMLAttr{name: String::from("a1"), value: 42.to_string()};
        let test_attr2 = XMLAttr{name: String::from("a2"), value: 24.to_string()};
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele1, point_ele2]),
            text: Some(String::from("some content")),
            attrs: Some(vec![test_attr1, test_attr2]),
        };

        assert_eq!(expected, ele2.to_string_pretty("\n","\t"));
    }

    #[test]
    fn xmlelement_to_string_pretty2() {
        let expected = format!(r#"<test_element a1="42" a2="24">{}<point lat="12.3" lon="45.6">{}point content{}</point>{}<point lat="32.1" lon="65.4"/>{}some content{}</test_element>"#, "\n\t", "\n\t\t","\n\t","\n\t", "\n\t", "\n");

        let mut point_ele1: XMLElement = Point{lat: 12.3, lon: 45.6}.into();
        point_ele1.set_text("point content");
        let point_ele2: XMLElement = Point{lat: 32.1, lon: 65.4}.into();
        let test_attr1 = XMLAttr{name: String::from("a1"), value: 42.to_string()};
        let test_attr2 = XMLAttr{name: String::from("a2"), value: 24.to_string()};
        let ele2 = XMLElement{
            name: String::from("test_element"),
            contents: Some(vec![point_ele1, point_ele2]),
            text: Some(String::from("some content")),
            attrs: Some(vec![test_attr1, test_attr2]),
        };
        assert_eq!(expected, ele2.to_string_pretty("\n","\t"));
    }

    #[test]
    fn xmlelement_to_string_pretty_prolog() {
        let expected = format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}<test_element>{}some content{}</test_element>"#, "\n", "\n\t", "\n");

        let ele = XMLElement{
            name: String::from("test_element"),
            contents: None,
            text: Some(String::from("some content")),
            attrs: None,
        };

        assert_eq!(expected, ele.to_string_pretty_prolog("\n","\t"));
    }

    #[test]
    fn test_split_cdata() {
        let input = "<![CDATA[]]>";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[]]>"), String::from(""))));

        let input = "<![CDATA[]>";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "<![CDATA[]>");
        assert_eq!(opt_cdata, None);

        let input = "<![CDTA[]]>";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "<![CDTA[]]>");
        assert_eq!(opt_cdata, None);

        let input = "hello<![CDATA[]]>";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "hello");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[]]>"), String::from(""))));

        let input = "hello<![CDATA[]]>world";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "hello");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[]]>"), String::from("world"))));

        let input = "hello<![CDATA[world]]>";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "hello");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[world]]>"), String::from(""))));

        let input = "hello<![CDATA[wor]]>ld";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "hello");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[wor]]>"), String::from("ld"))));

        let input = "<![CDATA[]]>world";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[]]>"), String::from("world"))));

        let input = "<![CDATA[hello]]>world";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[hello]]>"), String::from("world"))));

        let input = "<![CDATA[hel]]>lo]]>world";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[hel]]>"), String::from("lo]]>world"))));

        let input = "<![CDATA[hel<![CDATA[lo]]>world";
        let (before_cdata, opt_cdata) = split_cdata(&input);
        assert_eq!(before_cdata, "");
        assert_eq!(opt_cdata, Some((String::from("<![CDATA[hel<![CDATA[lo]]>"), String::from("world"))));
    }
}