# oxc-graphql-parser

A spec-compliant, error-resilient GraphQL lexer and parser for Rust.

## Features

- Typed GraphQL concrete syntax tree based on the [October 2021 specification]
- Error-resilient lexing and parsing
- GraphQL schema and query parsing
- Standalone lexer API

## Installation

```bash
cargo add oxc-graphql-parser
```

Or add it manually:

```toml
[dependencies]
oxc-graphql-parser = "0.0.1"
```

The Cargo package name uses hyphens. Import it from Rust as `oxc_graphql_parser`.

## Usage

```rust
use oxc_graphql_parser::Parser;

let input = "union SearchResult = Photo | Person | Cat | Dog";
let parser = Parser::new(input);
let cst = parser.parse();

assert!(cst.errors().is_empty());
```

`Parser::parse` always returns a concrete syntax tree, even when lexing or
parsing reports errors. Check `cst.errors()` before walking the document:

```rust
use oxc_graphql_parser::Parser;

let input = "union SearchResult = Photo | Person | Cat | Dog";
let parser = Parser::new(input);
let cst = parser.parse();

assert_eq!(0, cst.errors().len());

let document = cst.document();
for definition in document.definitions() {
    println!("{definition:?}");
}
```

## Examples

The [examples directory] contains integrations for diagnostics and analysis:

- [using oxc-graphql-parser with ariadne to display error diagnostics]
- [using oxc-graphql-parser with annotate_snippets to display error diagnostics]
- [checking for unused variables]

### Get Field Names In An Object

```rust
use oxc_graphql_parser::{cst, Parser};

let input = "
type ProductDimension {
  size: String
  weight: Float @tag(name: \"hi from inventory value type field\")
}
";

let parser = Parser::new(input);
let cst = parser.parse();

assert_eq!(0, cst.errors().len());

let document = cst.document();
for definition in document.definitions() {
    if let cst::Definition::ObjectTypeDefinition(object_type) = definition {
        assert_eq!(object_type.name().unwrap().text(), "ProductDimension");

        for field in object_type.fields_definition().unwrap().field_definitions() {
            println!("{}", field.name().unwrap().text());
        }
    }
}
```

### Get Variables Used In A Query

```rust
use oxc_graphql_parser::{cst, Parser};

let input = "
query GraphQuery($graph_id: ID!, $variant: String) {
  service(id: $graph_id) {
    schema(tag: $variant) {
      document
    }
  }
}
";

let parser = Parser::new(input);
let cst = parser.parse();

assert_eq!(0, cst.errors().len());

let document = cst.document();
for definition in document.definitions() {
    if let cst::Definition::OperationDefinition(operation) = definition {
        assert_eq!(operation.name().unwrap().text(), "GraphQuery");

        let variables: Vec<String> = operation
            .variable_definitions()
            .iter()
            .flat_map(|definitions| definitions.variable_definitions())
            .filter_map(|definition| Some(definition.variable()?.text().to_string()))
            .collect();

        assert_eq!(
            variables.as_slice(),
            ["graph_id".to_string(), "variant".to_string()]
        );
    }
}
```

## Rust Versions

`oxc-graphql-parser` is tested on the latest stable version of Rust.
Older versions may or may not be compatible.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

[examples directory]: https://github.com/oxc-project/oxc-graphql-parser/tree/main/crates/oxc_graphql_parser/examples
[using oxc-graphql-parser with ariadne to display error diagnostics]: https://github.com/oxc-project/oxc-graphql-parser/blob/main/crates/oxc_graphql_parser/examples/ariadne.rs
[using oxc-graphql-parser with annotate_snippets to display error diagnostics]: https://github.com/oxc-project/oxc-graphql-parser/blob/main/crates/oxc_graphql_parser/examples/annotate_snippet.rs
[checking for unused variables]: https://github.com/oxc-project/oxc-graphql-parser/blob/main/crates/oxc_graphql_parser/examples/unused_vars.rs
[October 2021 specification]: https://spec.graphql.org/October2021
