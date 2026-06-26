use criterion::*;
use oxc_graphql_parser::Lexer;
use oxc_graphql_parser::ast;

fn parse_schema(schema: &str) {
    let parser = oxc_graphql_parser::Parser::new(schema);
    let tree = parser.parse();
    let errors = tree.errors().collect::<Vec<_>>();

    if !errors.is_empty() {
        panic!("error parsing query: {errors:?}");
    }

    let document = tree.document();

    // Simulate a basic field traversal operation.
    for definition in &document.definitions {
        if let ast::Definition::ObjectType(operation) = definition {
            for field in &operation.fields {
                std::hint::black_box(&field.ty);
            }
        }
    }
}

fn bench_supergraph_parser(c: &mut Criterion) {
    let schema =
        include_str!("../crates/oxc_graphql_parser/test_data/parser/ok/0032_supergraph.graphql");

    c.bench_function("supergraph_parser", move |b| b.iter(|| parse_schema(schema)));
}

fn bench_supergraph_lexer(c: &mut Criterion) {
    let schema =
        include_str!("../crates/oxc_graphql_parser/test_data/parser/ok/0032_supergraph.graphql");

    c.bench_function("supergraph_lexer", move |b| {
        b.iter(|| {
            let lexer = Lexer::new(schema);

            for token_res in lexer {
                std::hint::black_box(token_res.unwrap());
            }
        })
    });
}

criterion_group!(benches, bench_supergraph_lexer, bench_supergraph_parser);
criterion_main!(benches);
