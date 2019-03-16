use simple_xml_serialize::XMLElement;
use simple_xml_serialize_macro::xml_element;

#[test]
fn code_gen_test_eq() {
    
    #[xml_element("custom_name_here")]
    struct Point {
        #[sxs_type_attr(rename="latitude")]
        lat: f32,
        #[sxs_type_attr]
        lon: f32,
        #[sxs_type_text]
        date: String,
    }

    struct Point2 {
        lat: f32,
        lon: f32,
        date: String,
    }

    impl From<&Point2> for XMLElement {
        fn from(p: &Point2) -> Self {
            XMLElement::new("custom_name_here")
                        .attr("latitude", p.lat)
                        .attr("lon", p.lon)
                        .text(&p.date)
        }
    }

    let my_point = Point {
        lat: 43.38,
        lon: 60.11,
        date: "25 Dec 2018".to_string(),
    };
    let my_point2 = Point2 {
        lat: 43.38,
        lon: 60.11,
        date: "25 Dec 2018".to_string(),
    };

    assert_eq!(XMLElement::from(&my_point), XMLElement::from(&my_point2));
    assert_eq!(XMLElement::from(&my_point).to_string(), XMLElement::from(&my_point2).to_string());
}

#[test]
fn code_gen_test_complex_1() {
    
    #[xml_element("custom_name_here")]
    struct Point {
        #[sxs_type_attr(rename="latitude")]
        lat: f32,
        #[sxs_type_attr]
        lon: f32,
        #[sxs_type_empty_attr]
        active: bool,
        #[sxs_type_text]
        date: String,
        #[sxs_type_element(rename="Identifier")]
        name: Name1
    }

    #[xml_element("Name")]
    struct Name1 {
        #[sxs_type_text]
        val: String,
    }

    struct Point2 {
        lat: f32,
        lon: f32,
        date: String,
        active: bool,
        identifier: Name2,
    }

    struct Name2 {
        val: String,
    }

    impl From<&Name2> for XMLElement {
        fn from(n: &Name2) -> Self {
            XMLElement::new("Identifier")
                        .text(&n.val)
        }
    }

    impl From<&Point2> for XMLElement {
        fn from(p: &Point2) -> Self {
            XMLElement::new("custom_name_here")
                        .attr("latitude", p.lat)
                        .attr("lon", p.lon)
                        .empty_attr(p.active)
                        .text(&p.date)
                        .element(&p.identifier)
        }
    }

    let my_point = Point {
        lat: 43.38,
        lon: 60.11,
        date: "25 Dec 2018".to_string(),
        active: true,
        name: Name1{val: "p1".to_string()},
    };
    let my_point2 = Point2 {
        lat: 43.38,
        lon: 60.11,
        date: "25 Dec 2018".to_string(),
        active: true,
        identifier: Name2{val: "p1".to_string()},
    };

    assert_eq!(XMLElement::from(&my_point), XMLElement::from(&my_point2));
    assert_eq!(XMLElement::from(&my_point).to_string(), XMLElement::from(&my_point2).to_string());
}

#[test]
fn code_gen_test_basic_1() {
    
    #[xml_element("Employee")]
    struct Person1 {
        #[sxs_type_element(rename="Name")]
        name: Name1,
    }

    #[xml_element("Name")]
    struct Name1 {
        #[sxs_type_text]
        val: String,
    }


    struct Person2 {
        name: Name2,
    }
    struct Name2 {
        val: String,
    }

    impl From<&Name2> for XMLElement {
        fn from(n: &Name2) -> Self {
            XMLElement::new("Name")
                        .text(&n.val)
        }
    }

    impl From<&Person2> for XMLElement {
        fn from(p: &Person2) -> Self {
            XMLElement::new("Employee")
                        .element(&p.name)
        }
    }

    let name1 = Name1{val: "Robert".to_string()};
    let person1 = Person1{name: name1};

    let name2 = Name2{val: "Robert".to_string()};
    let person2 = Person2{name: name2};


    assert_eq!(XMLElement::from(&person1), XMLElement::from(&person2));
    assert_eq!(XMLElement::from(&person1).to_string(), XMLElement::from(&person2).to_string());
}

#[test]
fn code_gen_test_basic_2() {
    
    #[xml_element("Employees")]
    struct Person1 {
        #[sxs_type_multi_element(rename="Name")]
        names: Vec<Name1>,
    }

    #[xml_element("Name")]
    struct Name1 {
        #[sxs_type_text]
        val: String,
    }


    struct Person2 {
        names: Vec<Name2>,
    }
    struct Name2 {
        val: String,
    }

    impl From<&Name2> for XMLElement {
        fn from(n: &Name2) -> Self {
            XMLElement::new("Name")
                        .text(&n.val)
        }
    }

    impl From<&Person2> for XMLElement {
        fn from(p: &Person2) -> Self {
            XMLElement::new("Employees")
                        .elements(&p.names)
        }
    }

    let name1_1 = Name1{val: "Alice".to_string()};
    let name1_2 = Name1{val: "Bob".to_string()};
    let name_vec = vec![name1_1, name1_2];
    let person1 = Person1{names: name_vec};

    let name2_1 = Name2{val: "Alice".to_string()};
    let name2_2 = Name2{val: "Bob".to_string()};
    let name_vec = vec![name2_1, name2_2];
    let person2 = Person2{names: name_vec};


    assert_eq!(XMLElement::from(&person1), XMLElement::from(&person2));
    assert_eq!(XMLElement::from(&person1).to_string(), XMLElement::from(&person2).to_string());
}

#[test]
fn code_gen_test_basic_3() {
    
    #[xml_element("Employee")]
    struct Person1 {
        #[sxs_type_attr(rename="Name")]
        name: String,
    }

    struct Person2 {
        name: String,
    }

    impl From<&Person2> for XMLElement {
        fn from(p: &Person2) -> Self {
            XMLElement::new("Employee")
                        .attr("Name", &p.name)
        }
    }

    let person1 = Person1{name: "Robert".to_string()};

    let person2 = Person2{name: "Robert".to_string()};


    assert_eq!(XMLElement::from(&person1), XMLElement::from(&person2));
    assert_eq!(XMLElement::from(&person1).to_string(), XMLElement::from(&person2).to_string());
}

#[test]
fn code_gen_test_basic_4() {
    
    #[xml_element("Employee")]
    struct Person1 {
        #[sxs_type_empty_attr]
        name: String,
    }

    struct Person2 {
        name: String,
    }

    impl From<&Person2> for XMLElement {
        fn from(p: &Person2) -> Self {
            XMLElement::new("Employee")
                        .empty_attr(&p.name)
        }
    }

    let person1 = Person1{name: "Robert".to_string()};

    let person2 = Person2{name: "Robert".to_string()};


    assert_eq!(XMLElement::from(&person1), XMLElement::from(&person2));
    assert_eq!(XMLElement::from(&person1).to_string(), XMLElement::from(&person2).to_string());
}

#[test]
fn code_gen_test_basic_5() {
    
    #[xml_element("Employee")]
    struct Person1 {
        #[sxs_type_text]
        name: String,
    }

    struct Person2 {
        name: String,
    }

    impl From<&Person2> for XMLElement {
        fn from(p: &Person2) -> Self {
            XMLElement::new("Employee")
                        .text(&p.name)
        }
    }

    let person1 = Person1{name: "Robert".to_string()};

    let person2 = Person2{name: "Robert".to_string()};

    assert_eq!(XMLElement::from(&person1), XMLElement::from(&person2));
    assert_eq!(XMLElement::from(&person1).to_string(), XMLElement::from(&person2).to_string());
}