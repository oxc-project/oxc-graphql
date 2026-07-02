use crate::Allocator;
use crate::Lexer;
use crate::Parser;
use crate::TokenKind;
use crate::ast;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[test]
fn lexer_tests() {
    let source = r#"
type Query {
  hello(name: String = "world"): String
}
"#;
    let (tokens, errors) = Lexer::new(source).lex();
    assert!(errors.is_empty());
    assert!(tokens.iter().any(|token| token.kind() == TokenKind::Name && token.data() == "Query"));
}

#[test]
fn parser_parses_object_type_definition() {
    let source = r#"
type Query {
  hello(name: String = "world"): String
}
"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();

    assert_eq!(ast.errors().len(), 0);
    let document = ast.document();
    assert_eq!(document.definitions.len(), 1);

    let ast::Definition::ObjectType(object) = &document.definitions[0] else {
        panic!("expected object type definition");
    };
    assert_eq!(object.name.as_str(), "Query");
    assert_eq!(object.fields.len(), 1);
    assert_eq!(object.fields[0].name.as_str(), "hello");
    assert_eq!(object.fields[0].arguments[0].name.as_str(), "name");
}

#[test]
fn parser_parses_query_variables_and_used_variables() {
    let source = r#"
query GraphQuery($graph_id: ID!, $variant: String) {
  service(id: $graph_id) {
    schema(tag: $variant) {
      document
    }
  }
}
"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let ast::Definition::Operation(operation) = &ast.document().definitions[0] else {
        panic!("expected operation definition");
    };
    assert_eq!(operation.name.as_ref().unwrap().as_str(), "GraphQuery");
    assert_eq!(operation.variable_definitions.len(), 2);

    let mut used = Vec::new();
    collect_variables(operation.selection_set.as_ref().unwrap(), &mut used);
    assert_eq!(used, ["graph_id", "variant"]);
}

#[test]
fn parser_parses_selection_set_and_type_roots() {
    let allocator = Allocator::default();
    let selection = Parser::new(&allocator, "{ product { name } }").parse_selection_set();
    assert_eq!(selection.errors().len(), 0);
    assert_eq!(selection.field_set().selections.len(), 1);

    let ty = Parser::new(&allocator, "[String!]!").parse_type();
    assert_eq!(ty.errors().len(), 0);
    assert!(matches!(ty.ty(), ast::Type::NonNull(_)));
}

#[test]
fn parser_parses_legacy_fragment_variables() {
    let source = "fragment F($v: Int) on T { f(x: $v) }";
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).allow_legacy_fragment_variables(true).parse();
    assert_eq!(ast.errors().len(), 0);

    let ast::Definition::Fragment(fragment) = &ast.document().definitions[0] else {
        panic!("expected fragment definition");
    };
    assert_eq!(fragment.name.as_str(), "F");
    assert_eq!(fragment.variable_definitions.len(), 1);
    assert_eq!(fragment.variable_definitions[0].variable.name.as_str(), "v");
}

#[test]
fn parser_ok_fixtures_have_no_errors() {
    for path in graphql_files("parser/ok") {
        let source = fs::read_to_string(&path).unwrap();
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, &source).parse();
        let errors = ast.errors().collect::<Vec<_>>();
        assert!(errors.is_empty(), "{}: {errors:?}", path.display());
    }
}

#[test]
fn parser_err_fixtures_have_errors() {
    for path in graphql_files("parser/err") {
        let source = fs::read_to_string(&path).unwrap();
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, &source).parse();
        assert!(ast.errors().len() > 0, "{}", path.display());
    }
}

#[test]
#[ignore]
fn ecosystem_graphql_corpus_has_no_parse_errors() {
    let root = std::env::var_os("OXC_GRAPHQL_ECOSYSTEM_REPOS")
        .map(PathBuf::from)
        .expect("set OXC_GRAPHQL_ECOSYSTEM_REPOS to an ecosystem-ci repos directory");
    let mut files = Vec::new();
    collect_graphql_files(&root, &mut files);

    let mut failures = Vec::new();
    for path in &files {
        let source = fs::read_to_string(path).unwrap();
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, &source).parse();
        let errors = ast.errors().collect::<Vec<_>>();
        if !errors.is_empty() {
            failures.push(format!("{}: {errors:?}", path.display()));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} ecosystem GraphQL files failed to parse:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
}

fn collect_variables<'a>(selection_set: &'a ast::SelectionSet<'_>, output: &mut Vec<&'a str>) {
    for selection in &selection_set.selections {
        if let ast::Selection::Field(field) = selection {
            for argument in &field.arguments {
                collect_variable_value(argument.value.as_ref(), output);
            }
            if let Some(selection_set) = &field.selection_set {
                collect_variables(selection_set, output);
            }
        }
    }
}

fn collect_variable_value<'a>(value: Option<&'a ast::Value<'_>>, output: &mut Vec<&'a str>) {
    match value {
        Some(ast::Value::Variable(variable)) => output.push(variable.name.as_str()),
        Some(ast::Value::List(list)) => {
            for value in &list.values {
                collect_variable_value(Some(value), output);
            }
        }
        Some(ast::Value::Object(object)) => {
            for field in &object.fields {
                collect_variable_value(field.value.as_ref(), output);
            }
        }
        _ => {}
    }
}

fn graphql_files(path: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data").join(path);
    let mut files = fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "graphql"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn collect_graphql_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if matches!(name, ".git" | "node_modules" | "target") {
                continue;
            }
            collect_graphql_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| matches!(extension.to_str(), Some("gql" | "graphql")))
        {
            files.push(path);
        }
    }
    files.sort();
}
