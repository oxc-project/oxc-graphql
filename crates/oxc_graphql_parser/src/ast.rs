use crate::Error;
use crate::LimitTracker;
use std::fmt;
use std::slice::Iter;

pub use oxc_allocator::{Box as AstBox, Vec as AstVec};

/// A half-open byte range into the source text.
///
/// Offsets are `u32`: source texts are limited to 4 GiB (asserted by
/// [`crate::Parser::new`]), which halves the size of every AST node that
/// carries a span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, PartialEq)]
pub struct Ast<'a, T> {
    source: &'a str,
    root: T,
    errors: Vec<Error>,
    comments: Vec<Span>,
    recursion_limit: LimitTracker,
    token_limit: LimitTracker,
}

impl<'a, T> Ast<'a, T> {
    pub(crate) fn new(
        source: &'a str,
        root: T,
        errors: Vec<Error>,
        comments: Vec<Span>,
        recursion_limit: LimitTracker,
        token_limit: LimitTracker,
    ) -> Self {
        Self { source, root, errors, comments, recursion_limit, token_limit }
    }

    pub fn root(&self) -> &T {
        &self.root
    }

    pub fn into_root(self) -> T {
        self.root
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn errors(&self) -> Iter<'_, Error> {
        self.errors.iter()
    }

    /// Comment token spans in document order.
    ///
    /// GraphQL comments are always line comments: each span covers `#` through
    /// the end of the line (excluding the line terminator).
    ///
    /// NOTE: Only comments consumed while parsing are recorded.
    /// [`Parser::parse`] reads to the end of input, so it collects every comment in the source.
    /// Partial roots ([`Parser::parse_selection_set`], [`Parser::parse_type`])
    /// stop at the end of the root, so comments past it are not included.
    ///
    /// [`Parser::parse`]: crate::Parser::parse
    /// [`Parser::parse_selection_set`]: crate::Parser::parse_selection_set
    /// [`Parser::parse_type`]: crate::Parser::parse_type
    pub fn comments(&self) -> &[Span] {
        &self.comments
    }

    pub fn recursion_limit(&self) -> LimitTracker {
        self.recursion_limit
    }

    pub fn token_limit(&self) -> LimitTracker {
        self.token_limit
    }
}

impl<'a> Ast<'a, Document<'a>> {
    pub fn document(&self) -> &Document<'a> {
        self.root()
    }
}

impl<'a> Ast<'a, SelectionSet<'a>> {
    pub fn field_set(&self) -> &SelectionSet<'a> {
        self.root()
    }
}

impl<'a> Ast<'a, Type<'a>> {
    pub fn ty(&self) -> &Type<'a> {
        self.root()
    }
}

#[derive(Debug, PartialEq)]
pub struct Document<'a> {
    pub definitions: AstVec<'a, Definition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Definition<'a> {
    Operation(OperationDefinition<'a>),
    Fragment(FragmentDefinition<'a>),
    Directive(DirectiveDefinition<'a>),
    Schema(SchemaDefinition<'a>),
    SchemaExtension(SchemaExtension<'a>),
    ScalarType(ScalarTypeDefinition<'a>),
    ScalarTypeExtension(ScalarTypeExtension<'a>),
    ObjectType(ObjectTypeDefinition<'a>),
    ObjectTypeExtension(ObjectTypeExtension<'a>),
    InterfaceType(InterfaceTypeDefinition<'a>),
    InterfaceTypeExtension(InterfaceTypeExtension<'a>),
    UnionType(UnionTypeDefinition<'a>),
    UnionTypeExtension(UnionTypeExtension<'a>),
    EnumType(EnumTypeDefinition<'a>),
    EnumTypeExtension(EnumTypeExtension<'a>),
    InputObjectType(InputObjectTypeDefinition<'a>),
    InputObjectTypeExtension(InputObjectTypeExtension<'a>),
}

impl<'a> Definition<'a> {
    pub fn name(&self) -> Option<&Name<'a>> {
        match self {
            Self::Operation(definition) => definition.name.as_ref(),
            Self::Fragment(definition) => Some(&definition.name),
            Self::Directive(definition) => Some(&definition.name),
            Self::Schema(_) | Self::SchemaExtension(_) => None,
            Self::ScalarType(definition) => Some(&definition.name),
            Self::ScalarTypeExtension(definition) => Some(&definition.name),
            Self::ObjectType(definition) => Some(&definition.name),
            Self::ObjectTypeExtension(definition) => Some(&definition.name),
            Self::InterfaceType(definition) => Some(&definition.name),
            Self::InterfaceTypeExtension(definition) => Some(&definition.name),
            Self::UnionType(definition) => Some(&definition.name),
            Self::UnionTypeExtension(definition) => Some(&definition.name),
            Self::EnumType(definition) => Some(&definition.name),
            Self::EnumTypeExtension(definition) => Some(&definition.name),
            Self::InputObjectType(definition) => Some(&definition.name),
            Self::InputObjectTypeExtension(definition) => Some(&definition.name),
        }
    }

    /// The source span of the definition, whichever variant it is.
    ///
    /// When adding a new variant, remember to extend this match as well.
    pub fn span(&self) -> Span {
        match self {
            Self::Operation(definition) => definition.span,
            Self::Fragment(definition) => definition.span,
            Self::Directive(definition) => definition.span,
            Self::Schema(definition) => definition.span,
            Self::SchemaExtension(definition) => definition.span,
            Self::ScalarType(definition) => definition.span,
            Self::ScalarTypeExtension(definition) => definition.span,
            Self::ObjectType(definition) => definition.span,
            Self::ObjectTypeExtension(definition) => definition.span,
            Self::InterfaceType(definition) => definition.span,
            Self::InterfaceTypeExtension(definition) => definition.span,
            Self::UnionType(definition) => definition.span,
            Self::UnionTypeExtension(definition) => definition.span,
            Self::EnumType(definition) => definition.span,
            Self::EnumTypeExtension(definition) => definition.span,
            Self::InputObjectType(definition) => definition.span,
            Self::InputObjectTypeExtension(definition) => definition.span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name<'a> {
    pub value: &'a str,
    pub span: Span,
}

impl Name<'_> {
    pub fn as_str(&self) -> &str {
        self.value
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringValue<'a> {
    pub raw: &'a str,
    pub value: &'a str,
    pub block: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, PartialEq)]
pub struct OperationDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub operation_type: OperationType,
    pub name: Option<Name<'a>>,
    pub variable_definitions: AstVec<'a, VariableDefinition<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<&'a SelectionSet<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct FragmentDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub variable_definitions: AstVec<'a, VariableDefinition<'a>>,
    pub type_condition: NamedType<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<&'a SelectionSet<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct SelectionSet<'a> {
    pub selections: AstVec<'a, Selection<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Selection<'a> {
    Field(Field<'a>),
    FragmentSpread(FragmentSpread<'a>),
    InlineFragment(InlineFragment<'a>),
}

impl Selection<'_> {
    /// The source span of the selection, whichever variant it is.
    ///
    /// When adding a new variant, remember to extend this match as well.
    pub fn span(&self) -> Span {
        match self {
            Self::Field(selection) => selection.span,
            Self::FragmentSpread(selection) => selection.span,
            Self::InlineFragment(selection) => selection.span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub alias: Option<Name<'a>>,
    pub name: Name<'a>,
    pub arguments: AstVec<'a, Argument<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<&'a SelectionSet<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct FragmentSpread<'a> {
    pub name: Name<'a>,
    pub arguments: AstVec<'a, Argument<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct InlineFragment<'a> {
    pub type_condition: Option<NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<&'a SelectionSet<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct VariableDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub variable: Variable<'a>,
    pub ty: Option<Type<'a>>,
    pub default_value: Option<Value<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variable<'a> {
    pub name: Name<'a>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct Argument<'a> {
    pub name: Name<'a>,
    pub value: Option<Value<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct Directive<'a> {
    pub name: Name<'a>,
    pub arguments: AstVec<'a, Argument<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Variable(Variable<'a>),
    Int(IntValue<'a>),
    Float(FloatValue<'a>),
    String(StringValue<'a>),
    Boolean(BooleanValue),
    Null(NullValue),
    Enum(EnumValue<'a>),
    List(ListValue<'a>),
    Object(ObjectValue<'a>),
    Missing(Span),
}

impl Value<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::Variable(value) => value.span,
            Self::Int(value) => value.span,
            Self::Float(value) => value.span,
            Self::String(value) => value.span,
            Self::Boolean(value) => value.span,
            Self::Null(value) => value.span,
            Self::Enum(value) => value.name.span,
            Self::List(value) => value.span,
            Self::Object(value) => value.span,
            Self::Missing(span) => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntValue<'a> {
    pub raw: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FloatValue<'a> {
    pub raw: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BooleanValue {
    pub value: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NullValue {
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumValue<'a> {
    pub name: Name<'a>,
}

#[derive(Debug, PartialEq)]
pub struct ListValue<'a> {
    pub values: AstVec<'a, Value<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct ObjectValue<'a> {
    pub fields: AstVec<'a, ObjectField<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct ObjectField<'a> {
    pub name: Name<'a>,
    pub value: Option<Value<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Type<'a> {
    Named(NamedType<'a>),
    List(ListType<'a>),
    NonNull(NonNullType<'a>),
    Missing(Span),
}

impl Type<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::Named(value) => value.name.span,
            Self::List(value) => value.span,
            Self::NonNull(value) => value.span,
            Self::Missing(span) => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NamedType<'a> {
    pub name: Name<'a>,
}

#[derive(Debug)]
pub struct ListType<'a> {
    pub ty: AstBox<'a, Type<'a>>,
    pub span: Span,
}

impl PartialEq for ListType<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.ty.as_ref() == other.ty.as_ref() && self.span == other.span
    }
}

#[derive(Debug)]
pub struct NonNullType<'a> {
    pub ty: AstBox<'a, Type<'a>>,
    pub span: Span,
}

impl PartialEq for NonNullType<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.ty.as_ref() == other.ty.as_ref() && self.span == other.span
    }
}

#[derive(Debug, PartialEq)]
pub struct SchemaDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub root_operations: AstVec<'a, RootOperationTypeDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct SchemaExtension<'a> {
    pub directives: AstVec<'a, Directive<'a>>,
    pub root_operations: AstVec<'a, RootOperationTypeDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RootOperationTypeDefinition<'a> {
    pub operation_type: OperationType,
    pub named_type: NamedType<'a>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct DirectiveDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub arguments: AstVec<'a, InputValueDefinition<'a>>,
    pub repeatable: bool,
    pub locations: AstVec<'a, DirectiveLocation<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectiveLocation<'a> {
    pub name: &'a str,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct ScalarTypeDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct ScalarTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct ObjectTypeDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct ObjectTypeExtension<'a> {
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct InterfaceTypeDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct InterfaceTypeExtension<'a> {
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct UnionTypeDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub members: AstVec<'a, NamedType<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct UnionTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub members: AstVec<'a, NamedType<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct EnumTypeDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub values: AstVec<'a, EnumValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct EnumTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub values: AstVec<'a, EnumValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct EnumValueDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub value: EnumValue<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct InputObjectTypeDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, InputValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct InputObjectTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, InputValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct FieldDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub arguments: AstVec<'a, InputValueDefinition<'a>>,
    pub ty: Option<Type<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct InputValueDefinition<'a> {
    pub description: Option<&'a StringValue<'a>>,
    pub name: Name<'a>,
    pub ty: Option<Type<'a>>,
    pub default_value: Option<Value<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}
