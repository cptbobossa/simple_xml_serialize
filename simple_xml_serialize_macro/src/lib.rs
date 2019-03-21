/*!
This is a procedural macro library to enable simple xml serialization by annotating structs with attribute macros.
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

There is also a feature `process_options` to allow all the same code to work behind `Option` types. This feature is behind
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
*/

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use quote::TokenStreamExt;
// ref
// https://tinkering.xyz/introduction-to-proc-macros/
// https://github.com/tylerreisinger/cache-macro/blob/master/src/lib.rs
// https://doc.rust-lang.org/1.26.2/unstable-book/language-features/proc-macro.html
// https://stackoverflow.com/questions/46002861/how-do-i-generate-quotetokens-from-both-a-constant-value-and-a-collection-of
// https://docs.rs/quote/0.6.11/quote/macro.quote.html
// https://docs.rs/quote/0.6.11/quote/trait.ToTokens.html
// https://docs.rs/syn/0.14/syn/struct.Attribute.html
// https://stackoverflow.com/questions/42484062/how-do-i-process-enum-struct-field-attributes-in-a-procedural-macro/42526546
// https://stackoverflow.com/questions/49506485/how-to-provide-attributes-for-fields-for-struct-annotated-with-an-attribute-itse

#[proc_macro_attribute]
pub fn xml_element(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).expect("failed to parse input");
    
    //clone our item so we can check and alter its attributes
    let mut original_clone = item.clone();

    // assert that we need to have a name argument for the new XMLElement
    let args = attr.to_string();
    assert!(args.starts_with("\""), "`#[xml_element]` requires an argument of the form `#[xml_element(\"xml_element_name_here\")]`");

    // trim down to just the value
    let element_name = args.trim_matches(&['=', ' ', '"'][..]);

    // match item and only continue if it is a struct type
    match item {
        syn::Item::Struct(ref struct_item) => {
            return gen_impl_code(&element_name, &mut original_clone, struct_item);
        },
        _ => {
            assert!(false, "#[xml_element] may only be applied to structs");
        },
    }

    unreachable!();
}

/// function with hardcoded values to remove from the vec of struct field attributes
fn remove_our_attrs_from_item_fields(original_struct: syn::Item) -> syn::Item {
    let our_attrs = ["sxs_type_attr", "sxs_type_element", "sxs_type_text", "sxs_type_multi_element"];

    let mut original_struct_clone = original_struct.clone();

    for a in our_attrs.iter() {
        original_struct_clone = remove_attr_from_item(original_struct_clone, a);
    }
    original_struct_clone
}

/// dig into the fields attributes and remove the attributes we added to avoid 
/// compilation errors after code generation is done
fn remove_attr_from_item(original_struct: syn::Item, to_remove: &str) -> syn::Item {
    if let syn::Item::Struct(mut struct_item) = original_struct {
        if let syn::Fields::Named(ref mut fields) = struct_item.fields {
            for field in fields.named.iter_mut() {
                let index = field.attrs.iter().position(|a| {
                    match a.interpret_meta() {
                        Some(w) => {
                            match w {
                                syn::Meta::Word(i) => &i.to_string() == to_remove,
                                syn::Meta::List(ml) => &ml.ident.to_string() == to_remove,
                                _ => false,
                            }
                        },
                        _ => false,
                    }
                });
                if let Some(found_index) = index {
                    field.attrs.remove(found_index);
                }
            }
        }
        // this has to go here since our destructuring above moves the value
        return struct_item.into(); 
    }
    original_struct
}

// new_element_name is what our xml element will ultimately be called
// original_struct is the struct this macro was applied to, since that has to exist in the final code
// ast is the breakdown of the struct stuff by syn that we need to examine for the code generation
fn gen_impl_code(new_element_name: &str, original_struct: &mut syn::Item, ast: &syn::ItemStruct) -> TokenStream {
    let struct_ident = &ast.ident;

    // get the ident and name of the fields our attribute were applied to
    let attr_field_idents           = get_field_idents_of_attr_type(&ast.fields, "sxs_type_attr");
    let element_field_idents        = get_field_idents_of_attr_type(&ast.fields, "sxs_type_element");
    let multi_element_field_idents  = get_field_idents_of_attr_type(&ast.fields, "sxs_type_multi_element");
    let text_field_idents           = get_field_idents_of_attr_type(&ast.fields, "sxs_type_text");

    // since get_field_idents_of_attr_type returns a vec of tuple and we can't use that correctly in quote!
    // the following is just breaking up the tuples into separate vecs
    
    // generate the code for the From trait impl
    let from_impl = quote! {
        impl From<#struct_ident> for XMLElement {
            fn from(si: #struct_ident) -> Self {
                XMLElement::from(&si)
            }
        }
    };

    let add_attrs_code = gen_xml_attr_code(attr_field_idents);
    let add_elements_code = gen_xml_element_code(element_field_idents);
    let add_multi_elements_code = gen_xml_multi_element_code(multi_element_field_idents);
    let add_text_code = gen_xml_text_code(text_field_idents);

    // build out our From using #()* for repetition
    let from_ref_impl = quote! {
        impl From<&#struct_ident> for XMLElement {
            fn from(si: &#struct_ident) -> Self {
                let mut new_ele = XMLElement::new(#new_element_name);
                
                #add_attrs_code

                #add_elements_code

                #add_multi_elements_code

                #add_text_code

                new_ele
            }
        }
    };
    // remove our attrs so it doesn't screw up the generated code
    let original_struct_with_our_attrs_removed = remove_our_attrs_from_item_fields(original_struct.clone());

    // build up our final generate code and return it
    let gen = quote! {
        #original_struct_with_our_attrs_removed

        #from_ref_impl

        #from_impl
    };
    gen.into()
}

fn gen_xml_attr_code(attr_field_idents: Vec<(syn::Ident, String, bool, bool)>) -> quote::__rt::TokenStream {
    let attr_field_names: Vec<String>     = attr_field_idents.iter().map(|(_,b,_,_)|b.clone()).collect();
    let attr_idents:      Vec<syn::Ident> = attr_field_idents.iter().map(|(a,_,_,_)|a.clone()).collect();
    let attr_is_options:  Vec<bool>       = attr_field_idents.iter().map(|(_,_,_,d)|d.clone()).collect();
    
    let mut add_attrs_code = quote!();

    for i in 0..attr_is_options.len() {
        let attr_is_option = attr_is_options.get(i).unwrap();
        let attr_name = attr_field_names.get(i).unwrap();
        let attr_ident = attr_idents.get(i).unwrap();

        let attr_code = match attr_is_option {
            false => {
                quote! { new_ele.add_attr(#attr_name, &si.#attr_ident); }
            },
            true => {
                quote! {
                    if let Some(a) = &si.#attr_ident {
                        new_ele.add_attr(#attr_name, &a);
                    }
                }
            },
        };
        add_attrs_code.append_all(attr_code);
    }
    add_attrs_code
}

fn gen_xml_text_code(text_field_idents: Vec<(syn::Ident, String, bool, bool)>) -> quote::__rt::TokenStream {
    let text_idents:        Vec<syn::Ident> = text_field_idents.iter().map(|(a,_,_,_)|a.clone()).collect();
    let text_is_options:    Vec<bool>       = text_field_idents.iter().map(|(_,_,_,d)|d.clone()).collect();
    
    let mut add_texts_code = quote!();

    for i in 0..text_is_options.len() {
        let text_is_option = text_is_options.get(i).unwrap();
        let text_ident = text_idents.get(i).unwrap();

        let text_code = match text_is_option {
            false => {
                quote! { new_ele.set_text(&si.#text_ident); }
            },
            true => {
                quote! {
                    if let Some(a) = &si.#text_ident {
                        new_ele.set_text(&a);
                    }
                }
            },
        };
        add_texts_code.append_all(text_code);
    }
    add_texts_code
}

fn gen_xml_element_code(element_field_idents: Vec<(syn::Ident, String, bool, bool)>) -> quote::__rt::TokenStream {
    let element_names:          Vec<String>     = element_field_idents.iter().map(|(_,b,_,_)|b.clone()).collect();
    let element_renamed:        Vec<bool>       = element_field_idents.iter().map(|(_,_,c,_)|c.clone()).collect();
    let element_idents:         Vec<syn::Ident> = element_field_idents.iter().map(|(a,_,_,_)|a.clone()).collect();
    let element_is_options:     Vec<bool>       = element_field_idents.iter().map(|(_,_,_,d)|d.clone()).collect();
    
    let mut add_elements_code = quote!();

    for i in 0..element_is_options.len() {
        let element_is_option = element_is_options.get(i).unwrap();
        let element_name = element_names.get(i).unwrap();
        let element_was_renamed = element_renamed.get(i).unwrap();
        let element_ident = element_idents.get(i).unwrap();

        let element_code = match element_is_option {
            false => match element_was_renamed {
                false => quote! { new_ele.add_element(&si.#element_ident); },
                true => quote! { new_ele.add_element(XMLElement::from(&si.#element_ident).name(#element_name)); },
            },
            true => match element_was_renamed {
                false => quote! {
                    if let Some(a) = &si.#element_ident {
                        new_ele.add_element(a);
                    }
                },
                true => quote! { 
                    if let Some(a) = &si.#element_ident {
                        new_ele.add_element(XMLElement::from(a).name(#element_name));
                    }
                },
            }, 
        };
        add_elements_code.append_all(element_code);
    }
    add_elements_code
}

fn gen_xml_multi_element_code(multi_element_field_idents: Vec<(syn::Ident, String, bool, bool)>) -> quote::__rt::TokenStream {
    let multi_element_names:        Vec<String>     = multi_element_field_idents.iter().map(|(_,b,_,_)|b.clone()).collect();
    let multi_element_renamed:      Vec<bool>       = multi_element_field_idents.iter().map(|(_,_,c,_)|c.clone()).collect();
    let multi_element_idents:       Vec<syn::Ident> = multi_element_field_idents.iter().map(|(a,_,_,_)|a.clone()).collect();
    let multi_element_is_options:   Vec<bool>       = multi_element_field_idents.iter().map(|(_,_,_,d)|d.clone()).collect();
    
    let mut add_multi_elements_code = quote!();

    for i in 0..multi_element_is_options.len() {
        let multi_element_is_option = multi_element_is_options.get(i).unwrap();
        let multi_element_name = multi_element_names.get(i).unwrap();
        let multi_element_was_renamed = multi_element_renamed.get(i).unwrap();
        let multi_element_ident = multi_element_idents.get(i).unwrap();

        let multi_element_code = match multi_element_is_option {
            false => match multi_element_was_renamed {
                false => quote! { 
                    new_ele.add_elements(&si.#multi_element_ident);
                },
                true => quote! { 
                    new_ele.add_elements_with_name(#multi_element_name, &si.#multi_element_ident); 
                },
            },
            true => match multi_element_was_renamed {
                false => quote! {
                    if let Some(a) = &si.#multi_element_ident {
                        new_ele.add_elements(a);
                    }
                },
                true => quote! { 
                    if let Some(a) = &si.#multi_element_ident {
                        new_ele.add_elements_with_name(#multi_element_name, a); 
                    }
                },
            }, 
        };
        add_multi_elements_code.append_all(multi_element_code);
    }
    add_multi_elements_code
}

// dig down into the attributes of the named fields of our struct.
// return the field idents that match the provided attr_type paired with the name they will 
// ultimately be serialized with and a bool specifying if we renamed the field or not
#[cfg(feature = "process_options")]
fn get_field_idents_of_attr_type(fields: &syn::Fields, attr_type: &str) -> Vec<(syn::Ident, String, bool, bool)> {
    match fields {
        syn::Fields::Named(ref fields) => {
            let mut field_vec = Vec::new();
            for field in &fields.named {
                for a in field.attrs.clone().iter() {
                    if let Some(w) = a.interpret_meta() {
                        match w {
                            // this is if our attribute is of the form #[sxs_type_element]
                            syn::Meta::Word(i) => {
                                if &i.to_string() == attr_type {
                                    if field.ident.is_some() {
                                        let is_option = is_option_type(&field.ty);
                                        // field.ident.to_string() gives us the name of the field
                                        let val = (field.clone().ident.unwrap(), field.clone().ident.unwrap().to_string(), false, is_option);
                                        field_vec.push(val);
                                    }
                                }
                            },
                            // this is if our attribute is of the form #[sxs_type_element(rename="new_name"))]
                            syn::Meta::List(ref ml) => {
                                let newname = extract_ident_with_new_name(ml, attr_type);
                                if newname.is_some() &&  field.ident.is_some(){
                                    let is_option = is_option_type(&field.ty);
                                    let fc = field.clone();
                                    field_vec.push((fc.ident.unwrap(), newname.unwrap(), true, is_option));
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
            field_vec
        }
        // Ignore unit structs or anonymous fields.
        _ => {
            Vec::new()
        },
    }
}

#[cfg(not(feature = "process_options"))]
fn get_field_idents_of_attr_type(fields: &syn::Fields, attr_type: &str) -> Vec<(syn::Ident, String, bool, bool)> {
    match fields {
        syn::Fields::Named(ref fields) => {
            let mut field_vec = Vec::new();
            for field in &fields.named {
                for a in field.attrs.clone().iter() {
                    if let Some(w) = a.interpret_meta() {
                        match w {
                            // this is if our attribute is of the form #[sxs_type_element]
                            syn::Meta::Word(i) => {
                                if &i.to_string() == attr_type {
                                    if field.ident.is_some() {
                                        // field.ident.to_string() gives us the name of the field
                                        let val = (field.clone().ident.unwrap(), field.clone().ident.unwrap().to_string(), false, false);
                                        field_vec.push(val);
                                    }
                                }
                            },
                            // this is if our attribute is of the form #[sxs_type_element(rename="new_name"))]
                            syn::Meta::List(ref ml) => {
                                let newname = extract_ident_with_new_name(ml, attr_type);
                                if newname.is_some() &&  field.ident.is_some(){
                                    let fc = field.clone();
                                    field_vec.push((fc.ident.unwrap(), newname.unwrap(), true, false));
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
            field_vec
        }
        // Ignore unit structs or anonymous fields.
        _ => {
            Vec::new()
        },
    }
}

/// digs down into `#[sxs_type_element(rename="new_name"))]` to grab "new_name"
fn extract_ident_with_new_name(ml: &syn::MetaList, attr_type: &str) -> Option<String> {
    if ml.ident.to_string() != attr_type {
        return None;
    }
    for nested in &ml.nested {
        if let syn::NestedMeta::Meta(nv) = nested {
            if let syn::Meta::NameValue(mnv) = nv {
                // the only type of attribute param we currently allow is "rename"
                if &mnv.ident.to_string() == "rename" {
                    if let syn::Lit::Str(ref ls) = mnv.lit {
                        return Some(ls.value());
                    }
                }
            }
        }
    }
    None
}

#[cfg(feature = "process_options")]
fn is_option_type(ty: &syn::Type) -> bool {

    // syn::Type::Path(TypePath)
    if let syn::Type::Path(t) = ty {
        if t.path.segments.len() > 0 {
            let path_seg = &t.path.segments[0];
            if path_seg.ident.to_string() == "Option" {
                return true;
            }
        }
    }

    false
}