use simple_xml_serialize::XMLElement;
use criterion::{criterion_group,criterion_main,Criterion,Benchmark};

fn new_element_1() -> String {
    let mut ele = XMLElement::new("person");
    ele.add_attr("age", 28); 
    ele.set_text("John Doe");
    ele.to_string()
}
fn new_element_2() -> String {
    XMLElement::new("person")
                    .attr("age", 28)
                    .text("John Doe").to_string()
}

fn bench_new_elements(c: &mut Criterion) {
    c.bench("Create Element",
        Benchmark::new("Setter", |b| b.iter(|| {
            new_element_1()
        })).with_function("Builder", |b| b.iter(|| {
            new_element_2()
        })),
    );
}

criterion_group!(benches, bench_new_elements);
criterion_main!(benches);