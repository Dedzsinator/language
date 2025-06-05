use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// The main AST node trait
pub trait AstNode {
    fn span(&self) -> &Span;
}

/// Top-level program structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Top-level items in the program
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    StructDef(StructDef),
    TypeclassDef(TypeclassDef),
    InstanceDef(InstanceDef),
    FunctionDef(FunctionDef),
    LetBinding(LetBinding),
    Import(Import),
}

/// Struct definition with optional fields and defaults
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub type_annotation: Type,
    pub optional: bool,
    pub default_value: Option<Expression>,
    pub span: Span,
}

/// Typeclass definition (Haskell-style)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeclassDef {
    pub name: String,
    pub type_param: String,
    pub methods: Vec<TypeclassMethod>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeclassMethod {
    pub name: String,
    pub type_signature: Type,
    pub span: Span,
}

/// Typeclass instance implementation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceDef {
    pub typeclass_name: String,
    pub type_name: String,
    pub implementations: Vec<MethodImpl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodImpl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Expression,
    pub span: Span,
}

/// Function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Expression,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Type,
    pub span: Span,
}

/// Function/variable attributes (@gpu, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

/// Let binding (variable/function definition)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetBinding {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub value: Expression,
    pub span: Span,
}

/// Import statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub module_path: String,
    pub items: Option<Vec<String>>, // None for wildcard import
    pub span: Span,
}

/// Type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types
    Int,
    Float,
    Bool,
    String,
    Unit,

    /// Composite types
    Struct(String),
    Array(Box<Type>),
    Matrix(Box<Type>, Option<usize>, Option<usize>), // element type, rows, cols
    Function(Vec<Type>, Box<Type>),                  // params -> return

    /// Generic types
    TypeVar(String),
    TypeApp(String, Vec<Type>), // Generic type application

    /// Optional type
    Option(Box<Type>),

    /// Span information
    Spanned(Box<Type>, Span),
}

/// Expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // Literals
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    BoolLiteral(bool, Span),
    StringLiteral(String, Span),

    // Identifiers
    Identifier(String, Span),

    // Structures
    StructCreation {
        name: String,
        fields: HashMap<String, Expression>,
        span: Span,
    },

    // Arrays and matrices
    ArrayLiteral(Vec<Expression>, Span),
    MatrixLiteral(Vec<Vec<Expression>>, Span),

    // Matrix comprehensions: [[expr | var in range] | var in range]
    MatrixComprehension {
        element: Box<Expression>,
        generators: Vec<Generator>,
        span: Span,
    },

    // Function calls
    FunctionCall {
        function: Box<Expression>,
        args: Vec<Expression>,
        span: Span,
    },

    // Lambda expressions
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expression>,
        span: Span,
    },

    // Binary operations
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },

    // Unary operations
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },

    // Field access
    FieldAccess {
        object: Box<Expression>,
        field: String,
        span: Span,
    },

    // Optional field access with fallback
    OptionalAccess {
        object: Box<Expression>,
        field: String,
        fallback: Box<Expression>,
        span: Span,
    },

    // Control flow
    IfExpression {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
        span: Span,
    },

    // Pattern matching
    Match {
        expression: Box<Expression>,
        arms: Vec<MatchArm>,
        span: Span,
    },

    // Let expressions
    Let {
        bindings: Vec<LetBinding>,
        body: Box<Expression>,
        span: Span,
    },

    // Block expressions
    Block {
        statements: Vec<Statement>,
        result: Option<Box<Expression>>,
        span: Span,
    },

    // Parallel execution
    Parallel {
        expressions: Vec<Expression>,
        span: Span,
    },

    // Spawn async computation
    Spawn {
        expression: Box<Expression>,
        span: Span,
    },

    // Wait for async result
    Wait {
        expression: Box<Expression>,
        span: Span,
    },

    // GPU execution directive
    GpuDirective {
        expression: Box<Expression>,
        span: Span,
    },

    // Range expressions (1..10, 1..=10)
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
        span: Span,
    },
}

/// Comprehension generators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generator {
    pub variable: String,
    pub iterable: Expression,
    pub condition: Option<Expression>, // Optional filter
    pub span: Span,
}

/// Match arms for pattern matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
    pub span: Span,
}

/// Patterns for pattern matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard(Span),
    Identifier(String, Span),
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    BoolLiteral(bool, Span),
    StringLiteral(String, Span),

    // Option patterns
    Some(Box<Pattern>, Span),
    None(Span),

    // Struct patterns
    Struct {
        name: String,
        fields: HashMap<String, Pattern>,
        span: Span,
    },

    // Array patterns
    Array(Vec<Pattern>, Span),
}

/// Statements (for block expressions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Expression(Expression),
    LetBinding(LetBinding),
}

/// Binary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // Matrix operations
    MatMul,       // Matrix multiplication
    DotProduct,   // Dot product
    CrossProduct, // Cross product

    // Optional chaining
    OptionalOr, // ??
}

/// Unary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Neg,       // -
    Not,       // !
    Transpose, // T (for matrices)
}

impl AstNode for Program {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl AstNode for StructDef {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl AstNode for TypeclassDef {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl AstNode for InstanceDef {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl AstNode for FunctionDef {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl AstNode for LetBinding {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl AstNode for Item {
    fn span(&self) -> &Span {
        match self {
            Item::StructDef(s) => &s.span,
            Item::TypeclassDef(t) => &t.span,
            Item::InstanceDef(i) => &i.span,
            Item::FunctionDef(f) => &f.span,
            Item::LetBinding(l) => &l.span,
            Item::Import(i) => &i.span,
        }
    }
}

impl Expression {
    pub fn span(&self) -> &Span {
        match self {
            Expression::IntLiteral(_, span) => span,
            Expression::FloatLiteral(_, span) => span,
            Expression::BoolLiteral(_, span) => span,
            Expression::StringLiteral(_, span) => span,
            Expression::Identifier(_, span) => span,
            Expression::StructCreation { span, .. } => span,
            Expression::ArrayLiteral(_, span) => span,
            Expression::MatrixLiteral(_, span) => span,
            Expression::MatrixComprehension { span, .. } => span,
            Expression::FunctionCall { span, .. } => span,
            Expression::Lambda { span, .. } => span,
            Expression::BinaryOp { span, .. } => span,
            Expression::UnaryOp { span, .. } => span,
            Expression::FieldAccess { span, .. } => span,
            Expression::OptionalAccess { span, .. } => span,
            Expression::IfExpression { span, .. } => span,
            Expression::Match { span, .. } => span,
            Expression::Let { span, .. } => span,
            Expression::Block { span, .. } => span,
            Expression::Parallel { span, .. } => span,
            Expression::Spawn { span, .. } => span,
            Expression::Wait { span, .. } => span,
            Expression::GpuDirective { span, .. } => span,
            Expression::Range { span, .. } => span,
        }
    }
}
