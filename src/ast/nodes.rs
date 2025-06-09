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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_span() -> Span {
        Span::new(0, 10, 1, 1)
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(5, 15, 2, 3);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 15);
        assert_eq!(span.line, 2);
        assert_eq!(span.column, 3);
    }

    #[test]
    fn test_program_creation() {
        let span = create_test_span();
        let program = Program {
            items: vec![],
            span: span.clone(),
        };
        assert_eq!(program.items.len(), 0);
        assert_eq!(program.span(), &span);
    }

    #[test]
    fn test_struct_def_creation() {
        let span = create_test_span();
        let field_span = create_test_span();

        let field = StructField {
            name: "x".to_string(),
            type_annotation: Type::Int,
            optional: false,
            default_value: None,
            span: field_span,
        };

        let struct_def = StructDef {
            name: "Point".to_string(),
            fields: vec![field],
            span: span.clone(),
        };

        assert_eq!(struct_def.name, "Point");
        assert_eq!(struct_def.fields.len(), 1);
        assert_eq!(struct_def.fields[0].name, "x");
        assert_eq!(struct_def.span(), &span);
    }

    #[test]
    fn test_function_def_creation() {
        let span = create_test_span();
        let param_span = create_test_span();
        let expr_span = create_test_span();

        let param = Parameter {
            name: "x".to_string(),
            type_annotation: Type::Int,
            span: param_span,
        };

        let body = Expression::IntLiteral(42, expr_span);

        let func_def = FunctionDef {
            name: "test_func".to_string(),
            params: vec![param],
            return_type: Some(Type::Int),
            body,
            attributes: vec![],
            span: span.clone(),
        };

        assert_eq!(func_def.name, "test_func");
        assert_eq!(func_def.params.len(), 1);
        assert_eq!(func_def.return_type, Some(Type::Int));
        assert_eq!(func_def.span(), &span);
    }

    #[test]
    fn test_expression_span_methods() {
        let span = create_test_span();

        // Test various expression types
        let int_expr = Expression::IntLiteral(42, span.clone());
        assert_eq!(int_expr.span(), &span);

        let float_expr = Expression::FloatLiteral(3.14, span.clone());
        assert_eq!(float_expr.span(), &span);

        let bool_expr = Expression::BoolLiteral(true, span.clone());
        assert_eq!(bool_expr.span(), &span);

        let string_expr = Expression::StringLiteral("test".to_string(), span.clone());
        assert_eq!(string_expr.span(), &span);

        let ident_expr = Expression::Identifier("x".to_string(), span.clone());
        assert_eq!(ident_expr.span(), &span);
    }

    #[test]
    fn test_binary_op_expression() {
        let span = create_test_span();
        let left = Box::new(Expression::IntLiteral(1, span.clone()));
        let right = Box::new(Expression::IntLiteral(2, span.clone()));

        let binary_expr = Expression::BinaryOp {
            left,
            operator: BinaryOperator::Add,
            right,
            span: span.clone(),
        };

        assert_eq!(binary_expr.span(), &span);
    }

    #[test]
    fn test_function_call_expression() {
        let span = create_test_span();
        let function = Box::new(Expression::Identifier("func".to_string(), span.clone()));
        let args = vec![Expression::IntLiteral(1, span.clone())];

        let call_expr = Expression::FunctionCall {
            function,
            args,
            span: span.clone(),
        };

        assert_eq!(call_expr.span(), &span);
    }

    #[test]
    fn test_struct_creation_expression() {
        let span = create_test_span();
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Expression::IntLiteral(1, span.clone()));

        let struct_expr = Expression::StructCreation {
            name: "Point".to_string(),
            fields,
            span: span.clone(),
        };

        assert_eq!(struct_expr.span(), &span);
    }

    #[test]
    fn test_array_literal_expression() {
        let span = create_test_span();
        let elements = vec![
            Expression::IntLiteral(1, span.clone()),
            Expression::IntLiteral(2, span.clone()),
        ];

        let array_expr = Expression::ArrayLiteral(elements, span.clone());
        assert_eq!(array_expr.span(), &span);
    }

    #[test]
    fn test_matrix_literal_expression() {
        let span = create_test_span();
        let rows = vec![
            vec![
                Expression::IntLiteral(1, span.clone()),
                Expression::IntLiteral(2, span.clone()),
            ],
            vec![
                Expression::IntLiteral(3, span.clone()),
                Expression::IntLiteral(4, span.clone()),
            ],
        ];

        let matrix_expr = Expression::MatrixLiteral(rows, span.clone());
        assert_eq!(matrix_expr.span(), &span);
    }

    #[test]
    fn test_if_expression() {
        let span = create_test_span();
        let condition = Box::new(Expression::BoolLiteral(true, span.clone()));
        let then_branch = Box::new(Expression::IntLiteral(1, span.clone()));
        let else_branch = Some(Box::new(Expression::IntLiteral(2, span.clone())));

        let if_expr = Expression::IfExpression {
            condition,
            then_branch,
            else_branch,
            span: span.clone(),
        };

        assert_eq!(if_expr.span(), &span);
    }

    #[test]
    fn test_match_expression() {
        let span = create_test_span();
        let expression = Box::new(Expression::IntLiteral(1, span.clone()));
        let pattern = Pattern::IntLiteral(1, span.clone());
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Expression::IntLiteral(42, span.clone()),
            span: span.clone(),
        };

        let match_expr = Expression::Match {
            expression,
            arms: vec![arm],
            span: span.clone(),
        };

        assert_eq!(match_expr.span(), &span);
    }

    #[test]
    fn test_patterns() {
        let span = create_test_span();

        let wildcard = Pattern::Wildcard(span.clone());
        let ident = Pattern::Identifier("x".to_string(), span.clone());
        let int_pat = Pattern::IntLiteral(42, span.clone());
        let float_pat = Pattern::FloatLiteral(3.14, span.clone());
        let bool_pat = Pattern::BoolLiteral(true, span.clone());
        let string_pat = Pattern::StringLiteral("test".to_string(), span.clone());

        // Just ensure they can be created without panicking
        assert!(matches!(wildcard, Pattern::Wildcard(_)));
        assert!(matches!(ident, Pattern::Identifier(_, _)));
        assert!(matches!(int_pat, Pattern::IntLiteral(42, _)));
        assert!(matches!(float_pat, Pattern::FloatLiteral(_, _)));
        assert!(matches!(bool_pat, Pattern::BoolLiteral(true, _)));
        assert!(matches!(string_pat, Pattern::StringLiteral(_, _)));
    }

    #[test]
    fn test_type_variants() {
        // Test all type variants can be created
        let types = vec![
            Type::Int,
            Type::Float,
            Type::Bool,
            Type::String,
            Type::Unit,
            Type::Struct("TestStruct".to_string()),
            Type::Array(Box::new(Type::Int)),
            Type::Matrix(Box::new(Type::Float), Some(3), Some(3)),
            Type::Function(vec![Type::Int], Box::new(Type::String)),
            Type::TypeVar("T".to_string()),
            Type::TypeApp("Option".to_string(), vec![Type::Int]),
            Type::Option(Box::new(Type::String)),
            Type::Spanned(Box::new(Type::Int), create_test_span()),
        ];

        // Ensure all types can be created and are different
        assert_eq!(types.len(), 13);
    }

    #[test]
    fn test_binary_operators() {
        let operators = vec![
            BinaryOperator::Add,
            BinaryOperator::Sub,
            BinaryOperator::Mul,
            BinaryOperator::Div,
            BinaryOperator::Mod,
            BinaryOperator::Pow,
            BinaryOperator::Eq,
            BinaryOperator::Ne,
            BinaryOperator::Lt,
            BinaryOperator::Le,
            BinaryOperator::Gt,
            BinaryOperator::Ge,
            BinaryOperator::And,
            BinaryOperator::Or,
            BinaryOperator::MatMul,
            BinaryOperator::DotProduct,
            BinaryOperator::CrossProduct,
            BinaryOperator::OptionalOr,
        ];

        assert_eq!(operators.len(), 18);
    }

    #[test]
    fn test_unary_operators() {
        let operators = vec![
            UnaryOperator::Neg,
            UnaryOperator::Not,
            UnaryOperator::Transpose,
        ];

        assert_eq!(operators.len(), 3);
    }

    #[test]
    fn test_let_binding() {
        let span = create_test_span();
        let value = Expression::IntLiteral(42, span.clone());

        let let_binding = LetBinding {
            name: "x".to_string(),
            type_annotation: Some(Type::Int),
            value,
            span: span.clone(),
        };

        assert_eq!(let_binding.name, "x");
        assert_eq!(let_binding.type_annotation, Some(Type::Int));
        assert_eq!(let_binding.span(), &span);
    }

    #[test]
    fn test_import() {
        let span = create_test_span();

        let import = Import {
            module_path: "std::io".to_string(),
            items: Some(vec!["print".to_string(), "println".to_string()]),
            span: span.clone(),
        };

        assert_eq!(import.module_path, "std::io");
        assert_eq!(import.items.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_item_span_delegation() {
        let span = create_test_span();

        let struct_def = StructDef {
            name: "Test".to_string(),
            fields: vec![],
            span: span.clone(),
        };

        let item = Item::StructDef(struct_def);
        assert_eq!(item.span(), &span);
    }

    #[test]
    fn test_parallel_expression() {
        let span = create_test_span();
        let expressions = vec![
            Expression::IntLiteral(1, span.clone()),
            Expression::IntLiteral(2, span.clone()),
        ];

        let parallel_expr = Expression::Parallel {
            expressions,
            span: span.clone(),
        };

        assert_eq!(parallel_expr.span(), &span);
    }

    #[test]
    fn test_async_expressions() {
        let span = create_test_span();
        let inner_expr = Box::new(Expression::IntLiteral(42, span.clone()));

        let spawn_expr = Expression::Spawn {
            expression: inner_expr.clone(),
            span: span.clone(),
        };

        let wait_expr = Expression::Wait {
            expression: inner_expr,
            span: span.clone(),
        };

        assert_eq!(spawn_expr.span(), &span);
        assert_eq!(wait_expr.span(), &span);
    }

    #[test]
    fn test_range_expression() {
        let span = create_test_span();
        let start = Box::new(Expression::IntLiteral(1, span.clone()));
        let end = Box::new(Expression::IntLiteral(10, span.clone()));

        let range_expr = Expression::Range {
            start,
            end,
            inclusive: false,
            span: span.clone(),
        };

        assert_eq!(range_expr.span(), &span);
    }
}
