use criterion::{black_box, criterion_group, criterion_main, Criterion};
use protobuf_to_zod::parser::parse_proto_file;

fn benchmark_parse_proto_file(c: &mut Criterion) {
    let sample_proto = r#"
        syntax = "proto3";
        package example;

        message Person {
            string name = 1;
            int32 age = 2;
            repeated string hobbies = 3;
        }

        enum Status {
            UNKNOWN = 0;
            ACTIVE = 1;
            INACTIVE = 2;
        }

        service GreetingService {
            rpc SayHello (Person) returns (Person);
        }
    "#;

    c.bench_function("parse_proto_file", |b| {
        b.iter(|| parse_proto_file(black_box(sample_proto)))
    });
}

criterion_group!(benches, benchmark_parse_proto_file);
criterion_main!(benches);
