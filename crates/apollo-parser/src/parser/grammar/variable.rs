use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::name;
use crate::parser::grammar::ty;
use crate::parser::grammar::value;
use crate::parser::grammar::value::Constness;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::S;
use crate::T;
use std::ops::ControlFlow;

/// See: https://spec.graphql.org/October2021/#VariableDefinitions
///
/// *VariableDefinitions*:
///     **(** VariableDefinition* **)**
pub(crate) fn variable_definitions(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
    p.bump(S!['(']);

    // A Variable Definition may start with a Description (`"..."`) only when
    // executable descriptions are enabled; otherwise it must start with `$`.
    let allow_desc = p.executable_descriptions_allowed();
    let starts_definition =
        |kind| matches!(kind, T![$]) || (allow_desc && matches!(kind, TokenKind::StringValue));

    if p.peek().is_some_and(starts_definition) {
        variable_definition(p);
    } else {
        p.err("expected a Variable Definition")
    }
    p.peek_while(|p, kind| {
        if starts_definition(kind) {
            variable_definition(p);
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    });

    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/draft/#VariableDefinition
///
/// *VariableDefinition*:
///     Description? Variable **:** Type DefaultValue? Directives[Const]?
///
/// The leading *Description* is only parsed when
/// [`Parser::allow_executable_descriptions`] is enabled.
pub(crate) fn variable_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::VARIABLE_DEFINITION);

    if p.executable_descriptions_allowed() {
        if let Some(TokenKind::StringValue) = p.peek() {
            description::description(p);
            if p.peek() != Some(T![$]) {
                return p.err("expected a Variable");
            }
        }
    }

    variable(p);

    if let Some(T![:]) = p.peek() {
        p.bump(S![:]);
        if let Some(TokenKind::Name | TokenKind::LBracket) = p.peek() {
            ty::ty(p);
            if let Some(T![=]) = p.peek() {
                value::default_value(p);
            }
            if let Some(T![@]) = p.peek() {
                directive::directives(p, Constness::Const)
            }
        } else {
            p.err("expected a Type");
        }
    } else {
        p.err("expected a Name");
    }
}

/// See: https://spec.graphql.org/October2021/#Variable
///
/// *Variable*:
///     **$** Name
pub(crate) fn variable(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE);
    p.bump(S![$]);
    name::name(p);
}

#[cfg(test)]
mod test {
    use crate::cst;
    use crate::Parser;

    #[test]
    fn it_accesses_variable_name_and_type() {
        let gql = r#"
query GroceryStoreTrip($budget: Int) {
    name
}
        "#;

        let parser = Parser::new(gql);
        let cst = parser.parse();

        assert!(cst.errors().len() == 0);

        let doc = cst.document();

        for definition in doc.definitions() {
            if let cst::Definition::OperationDefinition(op_def) = definition {
                for var in op_def
                    .variable_definitions()
                    .unwrap()
                    .variable_definitions()
                {
                    assert_eq!(
                        var.variable().unwrap().name().unwrap().text().as_ref(),
                        "budget"
                    );
                    if let cst::Type::NamedType(name) = var.ty().unwrap() {
                        assert_eq!(name.name().unwrap().text().as_ref(), "Int")
                    }
                }
            }
        }
    }

    #[test]
    fn it_parses_variable_definition_description_when_enabled() {
        let input = "query Q(\"Budget for the trip\" $budget: Int) { name }";
        let cst = Parser::new(input)
            .allow_executable_descriptions(true)
            .parse();

        assert_eq!(cst.errors().len(), 0);
        let def = cst.document().definitions().next().unwrap();
        let cst::Definition::OperationDefinition(op) = def else {
            panic!("expected an OperationDefinition");
        };
        let var = op
            .variable_definitions()
            .unwrap()
            .variable_definitions()
            .next()
            .unwrap();
        assert!(var.description().is_some());
    }
}
