# Architecture

`oxc-graphql-parser` is a hand-written, recursive-descent GraphQL lexer and
parser. It takes GraphQL source text and produces a typed AST plus a list of
lexing and parsing errors.

## Design Principles

1. **Developer experience.** The public API exposes ordinary Rust structs and
   enums that are easy to inspect, pattern-match, and pass to downstream tools.

2. **Stability and reliability.** The parser remains spec-oriented and
   error-resilient. Parsing returns an AST even when diagnostics are present.

3. **Diagnostics.** Lexing and parsing collect errors instead of returning
   early. AST nodes carry byte spans so diagnostics can point back into the
   original source.

4. **Extensibility.** The AST is suitable for schema diagnostics, composition,
   query planning, linting, and other GraphQL tooling.

## Lexer

The lexer is an iterator over `Result<Token, Error>`. It recognizes GraphQL
punctuators, names, numbers, strings, comments, commas, whitespace, and EOF.
Lexing errors are collected and parsing continues when possible.

## Parser

The parser consumes significant lexer tokens and skips GraphQL ignored tokens:
comments, whitespace, and commas. It builds the AST directly while preserving
node spans.

The main entry points are:

```rust
use oxc_graphql_parser::{Allocator, Parser};

let allocator = Allocator::default();
let document = Parser::new(&allocator, "query Q { viewer { id } }").parse();
assert_eq!(document.errors().len(), 0);

let selection_set = Parser::new(&allocator, "{ viewer { id } }").parse_selection_set();
assert_eq!(selection_set.errors().len(), 0);

let ty = Parser::new(&allocator, "[String!]!").parse_type();
assert_eq!(ty.errors().len(), 0);
```

## AST

The AST lives in `oxc_graphql_parser::ast`. It uses semantic Rust types such as
`Document`, `Definition`, `OperationDefinition`, `SelectionSet`, `Field`,
`Value`, and `Type`.

```rust
use oxc_graphql_parser::{Allocator, ast, Parser};

let allocator = Allocator::default();
let parsed = Parser::new(&allocator, "type Query { hello: String }").parse();
let document = parsed.document();

let ast::Definition::ObjectType(object) = &document.definitions[0] else {
    panic!("expected object type");
};

assert_eq!(object.name.as_str(), "Query");
assert_eq!(object.fields[0].name.as_str(), "hello");
```

This direct-AST design intentionally does not keep a lossless red/green syntax
tree. Comments, whitespace, and commas are not represented as AST nodes.
