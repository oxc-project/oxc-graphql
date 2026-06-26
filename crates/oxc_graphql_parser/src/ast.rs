use crate::Error;
use crate::LimitTracker;
use std::fmt;
use std::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ast<T> {
    source: String,
    root: T,
    errors: Vec<Error>,
    recursion_limit: LimitTracker,
    token_limit: LimitTracker,
}

impl<T> Ast<T> {
    pub(crate) fn new(
        source: &str,
        root: T,
        errors: Vec<Error>,
        recursion_limit: LimitTracker,
        token_limit: LimitTracker,
    ) -> Self {
        Self { source: source.to_string(), root, errors, recursion_limit, token_limit }
    }

    pub fn root(&self) -> &T {
        &self.root
    }

    pub fn into_root(self) -> T {
        self.root
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn errors(&self) -> Iter<'_, Error> {
        self.errors.iter()
    }

    pub fn recursion_limit(&self) -> LimitTracker {
        self.recursion_limit
    }

    pub fn token_limit(&self) -> LimitTracker {
        self.token_limit
    }
}

impl Ast<Document> {
    pub fn document(&self) -> &Document {
        self.root()
    }
}

impl Ast<SelectionSet> {
    pub fn field_set(&self) -> &SelectionSet {
        self.root()
    }
}

impl Ast<Type> {
    pub fn ty(&self) -> &Type {
        self.root()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub definitions: Vec<Definition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Operation(OperationDefinition),
    Fragment(FragmentDefinition),
    Directive(DirectiveDefinition),
    Schema(SchemaDefinition),
    SchemaExtension(SchemaExtension),
    ScalarType(ScalarTypeDefinition),
    ScalarTypeExtension(ScalarTypeExtension),
    ObjectType(ObjectTypeDefinition),
    ObjectTypeExtension(ObjectTypeExtension),
    InterfaceType(InterfaceTypeDefinition),
    InterfaceTypeExtension(InterfaceTypeExtension),
    UnionType(UnionTypeDefinition),
    UnionTypeExtension(UnionTypeExtension),
    EnumType(EnumTypeDefinition),
    EnumTypeExtension(EnumTypeExtension),
    InputObjectType(InputObjectTypeDefinition),
    InputObjectTypeExtension(InputObjectTypeExtension),
}

impl Definition {
    pub fn name(&self) -> Option<&Name> {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub value: String,
    pub span: Span,
}

impl Name {
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringValue {
    pub raw: String,
    pub value: String,
    pub block: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationDefinition {
    pub description: Option<StringValue>,
    pub operation_type: OperationType,
    pub name: Option<Name>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub variable_definitions: Vec<VariableDefinition>,
    pub type_condition: NamedType,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSet {
    pub selections: Vec<Selection>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    Field(Field),
    FragmentSpread(FragmentSpread),
    InlineFragment(InlineFragment),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Argument>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentSpread {
    pub name: Name,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefinition {
    pub description: Option<StringValue>,
    pub variable: Variable,
    pub ty: Option<Type>,
    pub default_value: Option<Value>,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    pub name: Name,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: Name,
    pub value: Option<Value>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub name: Name,
    pub arguments: Vec<Argument>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Variable(Variable),
    Int(IntValue),
    Float(FloatValue),
    String(StringValue),
    Boolean(BooleanValue),
    Null(NullValue),
    Enum(EnumValue),
    List(ListValue),
    Object(ObjectValue),
    Missing(Span),
}

impl Value {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntValue {
    pub raw: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloatValue {
    pub raw: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumValue {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListValue {
    pub values: Vec<Value>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectValue {
    pub fields: Vec<ObjectField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectField {
    pub name: Name,
    pub value: Option<Value>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(NamedType),
    List(ListType),
    NonNull(NonNullType),
    Missing(Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Self::Named(value) => value.name.span,
            Self::List(value) => value.span,
            Self::NonNull(value) => value.span,
            Self::Missing(span) => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedType {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListType {
    pub ty: Box<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonNullType {
    pub ty: Box<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDefinition {
    pub description: Option<StringValue>,
    pub directives: Vec<Directive>,
    pub root_operations: Vec<RootOperationTypeDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaExtension {
    pub directives: Vec<Directive>,
    pub root_operations: Vec<RootOperationTypeDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RootOperationTypeDefinition {
    pub operation_type: OperationType,
    pub named_type: NamedType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectiveDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub arguments: Vec<InputValueDefinition>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveLocation {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarTypeExtension {
    pub name: Name,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub interfaces: Vec<NamedType>,
    pub directives: Vec<Directive>,
    pub fields: Vec<FieldDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTypeExtension {
    pub name: Name,
    pub interfaces: Vec<NamedType>,
    pub directives: Vec<Directive>,
    pub fields: Vec<FieldDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub interfaces: Vec<NamedType>,
    pub directives: Vec<Directive>,
    pub fields: Vec<FieldDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceTypeExtension {
    pub name: Name,
    pub interfaces: Vec<NamedType>,
    pub directives: Vec<Directive>,
    pub fields: Vec<FieldDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub members: Vec<NamedType>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeExtension {
    pub name: Name,
    pub directives: Vec<Directive>,
    pub members: Vec<NamedType>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub values: Vec<EnumValueDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeExtension {
    pub name: Name,
    pub directives: Vec<Directive>,
    pub values: Vec<EnumValueDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValueDefinition {
    pub description: Option<StringValue>,
    pub value: EnumValue,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub fields: Vec<InputValueDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectTypeExtension {
    pub name: Name,
    pub directives: Vec<Directive>,
    pub fields: Vec<InputValueDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub arguments: Vec<InputValueDefinition>,
    pub ty: Option<Type>,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputValueDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub ty: Option<Type>,
    pub default_value: Option<Value>,
    pub directives: Vec<Directive>,
    pub span: Span,
}
