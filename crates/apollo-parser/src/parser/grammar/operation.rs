use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::name;
use crate::parser::grammar::selection;
use crate::parser::grammar::value::Constness;
use crate::parser::grammar::variable;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::T;

/// See: https://spec.graphql.org/draft/#OperationDefinition
///
/// *OperationDefinition*:
///    Description? OperationType Name? VariableDefinitions? Directives? SelectionSet
///    SelectionSet
///
/// The leading *Description* is only parsed when
/// [`Parser::allow_executable_descriptions`] is enabled (2025 draft spec,
/// accepted by graphql-js 16); otherwise a leading string is an error as in
/// October 2021.
pub(crate) fn operation_definition(p: &mut Parser) {
    // A leading Description is only recognized when executable descriptions are
    // enabled; otherwise a string falls through to the error arm, exactly as in
    // October 2021.
    let has_description =
        p.executable_descriptions_allowed() && matches!(p.peek(), Some(TokenKind::StringValue));

    match p.peek() {
        Some(TokenKind::Name) => {
            let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);
            named_operation_definition(p);
        }
        Some(T!['{']) => {
            let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);
            selection::selection_set(p)
        }
        Some(TokenKind::StringValue) if has_description => {
            let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);
            description::description(p);

            match p.peek() {
                Some(TokenKind::Name) => named_operation_definition(p),
                Some(T!['{']) => {
                    // The spec does not allow a Description on a shorthand
                    // (anonymous) operation; parse the selection set anyway.
                    p.err("a Description is not allowed on a shorthand operation");
                    selection::selection_set(p)
                }
                _ => p.err_and_pop("expected an Operation Type or a Selection Set"),
            }
        }
        _ => p.err_and_pop("expected an Operation Type or a Selection Set"),
    }
}

/// The part of an *OperationDefinition* starting at *OperationType*,
/// after any *Description* has been consumed.
fn named_operation_definition(p: &mut Parser) {
    operation_type(p);

    if let Some(TokenKind::Name) = p.peek() {
        name::name(p);
    }

    if let Some(T!['(']) = p.peek() {
        variable::variable_definitions(p)
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::NotConst);
    }

    match p.peek() {
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err_and_pop("expected a Selection Set"),
    }
}

/// See: https://spec.graphql.org/October2021/#OperationType
///
/// *OperationType*: one of
///    **query**    **mutation**    **subscription**
pub(crate) fn operation_type(p: &mut Parser) {
    if let Some(node) = p.peek_data() {
        let _g = p.start_node(SyntaxKind::OPERATION_TYPE);
        match node {
            "query" => p.bump(SyntaxKind::query_KW),
            "subscription" => p.bump(SyntaxKind::subscription_KW),
            "mutation" => p.bump(SyntaxKind::mutation_KW),
            _ => p.err_and_pop("expected either a 'mutation', a 'query', or a 'subscription'"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cst;
    use crate::Parser;

    // NOTE @lrlna: related PR to the spec to avoid this issue:
    // https://github.com/graphql/graphql-spec/pull/892
    #[test]
    fn it_continues_parsing_when_operation_definition_starts_with_description() {
        let input = "\"description\"{}";
        let parser = Parser::new(input);
        let cst = parser.parse();

        assert_eq!(cst.errors().len(), 2);
        assert_eq!(cst.document().definitions().count(), 1);
    }

    #[test]
    fn it_parses_operation_description_when_enabled() {
        let input = "\"Query description\"\nquery MyQuery { field }";
        let cst = Parser::new(input)
            .allow_executable_descriptions(true)
            .parse();

        assert_eq!(cst.errors().len(), 0);
        let def = cst.document().definitions().next().unwrap();
        let cst::Definition::OperationDefinition(op) = def else {
            panic!("expected an OperationDefinition");
        };
        assert!(op.description().is_some());
    }
}
