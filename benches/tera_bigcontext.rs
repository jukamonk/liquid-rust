#[macro_use]
extern crate serde_derive;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tera::{Context, Tera};

fn bench_big_loop_big_object(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        const NUM_OBJECTS: usize = 100;
        let mut objects = Vec::with_capacity(NUM_OBJECTS);
        for i in 0..NUM_OBJECTS {
            objects.push(BigObject::new(i));
        }

        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "big_loop.html",
            "
{%- for object in objects -%}
{{ object.field_a.i }}
{%- if object.field_a.i > 2 -%}
{%- break -%}
{%- endif -%}
{%- endfor -%}
",
        )])
        .unwrap();
        let mut context = Context::new();
        context.insert("objects", &objects);
        let rendering = tera.render("big_loop.html", &context).expect("Good render");
        assert_eq!(&rendering[..], "0123");
        b.iter(|| tera.render("big_loop.html", &context));
    });
    group.finish();
}

fn bench_macro_big_object(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let big_object = BigObject::new(1);
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            (
                "big_loop.html",
                "
{%- import \"macros.html\" as macros -%}
{%- for i in iterations -%}{{ macros::get_first(bo=big_object) }}{% endfor %}",
            ),
            (
                "macros.html",
                "{%- macro get_first(bo) -%}{{ bo.field_a.i }}{% endmacro get_first %}",
            ),
        ])
        .unwrap();
        let mut context = Context::new();
        context.insert("big_object", &big_object);
        context.insert("iterations", &(0..500).collect::<Vec<usize>>());
        let rendering = tera.render("big_loop.html", &context).expect("Good render");
        assert_eq!(rendering.len(), 500);
        assert_eq!(rendering.chars().next().expect("Char"), '1');
        b.iter(|| tera.render("big_loop.html", &context));
    });
    group.bench_function(BenchmarkId::new("render", "liquid"), |b| {
        const NUM_OBJECTS: usize = 100;
        let objects: Vec<_> = (0..NUM_OBJECTS).map(|i| {
            let data_wrapper= liquid::object!({
                "i": (i as i32),
                "v": "Meta
    Before we get to the details, two important notes about the ownership system.
    Rust has a focus on safety and speed. It accomplishes these goals through many ‘zero-cost abstractions’, which means that in Rust, abstractions cost as little as possible in order to make them work. The ownership system is a prime example of a zero cost abstraction. All of the analysis we’ll talk about in this guide is done at compile time. You do not pay any run-time cost for any of these features.
    However, this system does have a certain cost: learning curve. Many new users to Rust experience something we like to call ‘fighting with the borrow checker’, where the Rust compiler refuses to compile a program that the author thinks is valid. This often happens because the programmer’s mental model of how ownership should work doesn’t match the actual rules that Rust implements. You probably will experience similar things at first. There is good news, however: more experienced Rust developers report that once they work with the rules of the ownership system for a period of time, they fight the borrow checker less and less.
    With that in mind, let’s learn about borrowing.",
            });
            liquid::object!({
                "field_a": data_wrapper,
                "field_b": data_wrapper,
                "field_c": data_wrapper,
                "field_d": data_wrapper,
                "field_e": data_wrapper,
                "field_f": data_wrapper,
            })
        }).collect();
        let data = liquid::object!({
            "objects": objects,
        });

        let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
        let template = parser
            .parse(
                "
    {%- for object in objects -%}
    {{ object.field_a.i }}
    {%- if object.field_a.i > 2 -%}
    {%- break -%}
    {%- endif -%}
    {%- endfor -%}
    ",
            )
            .expect("Benchmark template parsing failed");

        template.render(&data).unwrap();
        b.iter(|| template.render(&data));
    });
    group.finish();
}

fn bench_macro_big_object_no_loop_with_set(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "no_loop.html",
            "
{% set many_fields=two_fields.a -%}
{{ many_fields.a }}
{{ many_fields.b }}
{{ many_fields.c }}
",
        )])
        .unwrap();
        let mut context = Context::new();
        context.insert("two_fields", &TwoFields::new());
        let rendering = tera.render("no_loop.html", &context).expect("Good render");
        assert_eq!(&rendering[..], "\nA\nB\nC\n");
        b.iter(|| tera.render("no_loop.html", &context));
    });
    group.finish();
}

fn bench_macro_big_object_no_loop_macro_call(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            (
                "macros.html",
                "
{%- macro show_a(many_fields) -%}
{{ many_fields.a }}
{%- endmacro show_a -%}
        ",
            ),
            (
                "no_loop.html",
                "{%- import \"macros.html\" as macros -%}
{{ macros::show_a(many_fields=two_fields.a) }}",
            ),
        ])
        .unwrap();
        let mut context = Context::new();
        context.insert("two_fields", &TwoFields::new());
        let rendering = tera.render("no_loop.html", &context).expect("Good render");
        assert_eq!(&rendering[..], "A");
        b.iter(|| tera.render("no_loop.html", &context));
    });
    group.finish();
}

#[derive(Serialize)]
struct BigObject {
    field_a: DataWrapper,
    field_b: DataWrapper,
    field_c: DataWrapper,
    field_d: DataWrapper,
    field_e: DataWrapper,
    field_f: DataWrapper,
}

impl BigObject {
    fn new(i: usize) -> BigObject {
        BigObject {
            field_a: DataWrapper::new(i),
            field_b: DataWrapper::new(i),
            field_c: DataWrapper::new(i),
            field_d: DataWrapper::new(i),
            field_e: DataWrapper::new(i),
            field_f: DataWrapper::new(i),
        }
    }
}

#[derive(Serialize)]
struct DataWrapper {
    i: usize,
    v: String,
}

impl DataWrapper {
    fn new(i: usize) -> DataWrapper {
        DataWrapper {
            i,
            v: "Meta
Before we get to the details, two important notes about the ownership system.
Rust has a focus on safety and speed. It accomplishes these goals through many ‘zero-cost abstractions’, which means that in Rust, abstractions cost as little as possible in order to make them work. The ownership system is a prime example of a zero cost abstraction. All of the analysis we’ll talk about in this guide is done at compile time. You do not pay any run-time cost for any of these features.
However, this system does have a certain cost: learning curve. Many new users to Rust experience something we like to call ‘fighting with the borrow checker’, where the Rust compiler refuses to compile a program that the author thinks is valid. This often happens because the programmer’s mental model of how ownership should work doesn’t match the actual rules that Rust implements. You probably will experience similar things at first. There is good news, however: more experienced Rust developers report that once they work with the rules of the ownership system for a period of time, they fight the borrow checker less and less.
With that in mind, let’s learn about borrowing.".into(),
        }
    }
}

#[derive(Serialize)]
struct TwoFields {
    a: ManyFields,
    b: String,
}

impl TwoFields {
    fn new() -> TwoFields {
        TwoFields {
            a: ManyFields::new(),
            b: "B".into(),
        }
    }
}

#[derive(Serialize)]
struct ManyFields {
    a: String,
    b: String,
    c: String,
    d: Vec<BigObject>,
    e: Vec<String>,
}

impl ManyFields {
    fn new() -> ManyFields {
        let mut d = Vec::new();
        for i in 0..500 {
            d.push(BigObject::new(i));
        }
        let mut e = Vec::new();
        for i in 0..100 {
            e.push(format!("This is String({})", i));
        }

        ManyFields {
            a: "A".into(),
            b: "B".into(),
            c: "C".into(),
            d,
            e,
        }
    }
}

criterion_group!(
    benches,
    bench_big_loop_big_object,
    bench_macro_big_object,
    bench_macro_big_object_no_loop_with_set,
    bench_macro_big_object_no_loop_macro_call,
);
criterion_main!(benches);
