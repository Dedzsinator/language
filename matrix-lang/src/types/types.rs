use crate::ast::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found} at line {line}, column {column}")]
    TypeMismatch {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    #[error("Unknown identifier: {name} at line {line}, column {column}")]
    UnknownIdentifier {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("Unknown type: {name} at line {line}, column {column}")]
    UnknownType {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("Field {field} not found in type {type_name} at line {line}, column {column}")]
    FieldNotFound {
        field: String,
        type_name: String,
        line: usize,
        column: usize,
    },

    #[error("Cannot call non-function type {type_name} at line {line}, column {column}")]
    NotCallable {
        type_name: String,
        line: usize,
        column: usize,
    },

    #[error("Wrong number of arguments: expected {expected}, found {found} at line {line}, column {column}")]
    WrongArgumentCount {
        expected: usize,
        found: usize,
        line: usize,
        column: usize,
    },

    #[error("Typeclass {typeclass} not implemented for type {type_name} at line {line}, column {column}")]
    TypeclassNotImplemented {
        typeclass: String,
        type_name: String,
        line: usize,
        column: usize,
    },

    #[error("Circular import detected: module {module} is already being imported")]
    CircularImport { module: String, chain: Vec<String> },
}

pub type TypeResult<T> = Result<T, TypeError>;

/// Inferred type with additional metadata
#[derive(Debug, Clone, PartialEq)]
pub struct InferredType {
    pub ty: Type,
    pub constraints: Vec<TypeConstraint>,
}

/// Type constraints for typeclass resolution
#[derive(Debug, Clone, PartialEq)]
pub struct TypeConstraint {
    pub typeclass: String,
    pub type_param: Type,
}

/// Type environment for variable bindings
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    pub bindings: HashMap<String, InferredType>,
    pub parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: TypeEnv) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn bind(&mut self, name: String, ty: InferredType) {
        self.bindings.insert(name, ty);
    }

    pub fn lookup(&self, name: &str) -> Option<&InferredType> {
        self.bindings
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }
}

/// Struct definitions registry
#[derive(Debug, Clone, Default)]
pub struct StructRegistry {
    pub structs: HashMap<String, StructDef>,
}

impl StructRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, struct_def: StructDef) {
        self.structs.insert(struct_def.name.clone(), struct_def);
    }

    pub fn get(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    pub fn has_field(&self, struct_name: &str, field_name: &str) -> bool {
        self.get(struct_name)
            .map(|s| s.fields.iter().any(|f| f.name == field_name))
            .unwrap_or(false)
    }

    pub fn get_field_type(&self, struct_name: &str, field_name: &str) -> Option<&Type> {
        self.get(struct_name)
            .and_then(|s| s.fields.iter().find(|f| f.name == field_name))
            .map(|f| &f.type_annotation)
    }

    pub fn is_field_optional(&self, struct_name: &str, field_name: &str) -> bool {
        self.get(struct_name)
            .and_then(|s| s.fields.iter().find(|f| f.name == field_name))
            .map(|f| f.optional)
            .unwrap_or(false)
    }
}

/// Typeclass definitions registry
#[derive(Debug, Clone, Default)]
pub struct TypeclassRegistry {
    pub typeclasses: HashMap<String, TypeclassDef>,
    pub instances: HashMap<(String, String), InstanceDef>, // (typeclass, type) -> instance
}

impl TypeclassRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_typeclass(&mut self, typeclass_def: TypeclassDef) {
        self.typeclasses
            .insert(typeclass_def.name.clone(), typeclass_def);
    }

    pub fn register_instance(&mut self, instance_def: InstanceDef) {
        let key = (
            instance_def.typeclass_name.clone(),
            instance_def.type_name.clone(),
        );
        self.instances.insert(key, instance_def);
    }

    pub fn get_typeclass(&self, name: &str) -> Option<&TypeclassDef> {
        self.typeclasses.get(name)
    }

    pub fn get_instance(&self, typeclass: &str, type_name: &str) -> Option<&InstanceDef> {
        self.instances
            .get(&(typeclass.to_string(), type_name.to_string()))
    }

    pub fn has_instance(&self, typeclass: &str, type_name: &str) -> bool {
        self.get_instance(typeclass, type_name).is_some()
    }

    pub fn get_method_type(&self, typeclass: &str, method: &str) -> Option<&Type> {
        self.get_typeclass(typeclass)
            .and_then(|tc| tc.methods.iter().find(|m| m.name == method))
            .map(|m| &m.type_signature)
    }
}

/// Type context containing all type information
#[derive(Debug, Clone)]
pub struct TypeContext {
    pub env: TypeEnv,
    pub structs: StructRegistry,
    pub typeclasses: TypeclassRegistry,
    pub next_type_var: usize,
}

impl TypeContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            env: TypeEnv::new(),
            structs: StructRegistry::new(),
            typeclasses: TypeclassRegistry::new(),
            next_type_var: 0,
        };

        // Register built-in types and typeclasses
        ctx.register_builtins();
        ctx
    }

    fn register_builtins(&mut self) {
        // Register built-in typeclasses

        // Addable typeclass
        let addable = TypeclassDef {
            name: "Addable".to_string(),
            type_param: "T".to_string(),
            methods: vec![TypeclassMethod {
                name: "+".to_string(),
                type_signature: Type::Function(
                    vec![
                        Type::TypeVar("T".to_string()),
                        Type::TypeVar("T".to_string()),
                    ],
                    Box::new(Type::TypeVar("T".to_string())),
                ),
                span: Span::new(0, 0, 0, 0),
            }],
            span: Span::new(0, 0, 0, 0),
        };
        self.typeclasses.register_typeclass(addable);

        // Register instances for built-in types
        let int_addable = InstanceDef {
            typeclass_name: "Addable".to_string(),
            type_name: "Int".to_string(),
            implementations: vec![MethodImpl {
                name: "+".to_string(),
                params: vec![
                    Parameter {
                        name: "a".to_string(),
                        type_annotation: Type::Int,
                        span: Span::new(0, 0, 0, 0),
                    },
                    Parameter {
                        name: "b".to_string(),
                        type_annotation: Type::Int,
                        span: Span::new(0, 0, 0, 0),
                    },
                ],
                body: Expression::Identifier(
                    "__builtin_add_int".to_string(),
                    Span::new(0, 0, 0, 0),
                ),
                span: Span::new(0, 0, 0, 0),
            }],
            span: Span::new(0, 0, 0, 0),
        };
        self.typeclasses.register_instance(int_addable);

        let float_addable = InstanceDef {
            typeclass_name: "Addable".to_string(),
            type_name: "Float".to_string(),
            implementations: vec![MethodImpl {
                name: "+".to_string(),
                params: vec![
                    Parameter {
                        name: "a".to_string(),
                        type_annotation: Type::Float,
                        span: Span::new(0, 0, 0, 0),
                    },
                    Parameter {
                        name: "b".to_string(),
                        type_annotation: Type::Float,
                        span: Span::new(0, 0, 0, 0),
                    },
                ],
                body: Expression::Identifier(
                    "__builtin_add_float".to_string(),
                    Span::new(0, 0, 0, 0),
                ),
                span: Span::new(0, 0, 0, 0),
            }],
            span: Span::new(0, 0, 0, 0),
        };
        self.typeclasses.register_instance(float_addable);

        // Register built-in functions
        self.register_builtin_functions();
    }

    fn register_builtin_functions(&mut self) {
        // Mathematical constants
        self.env.bind(
            "pi".to_string(),
            InferredType {
                ty: Type::Float,
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "e".to_string(),
            InferredType {
                ty: Type::Float,
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "tau".to_string(),
            InferredType {
                ty: Type::Float,
                constraints: Vec::new(),
            },
        );

        // print function: variadic function that takes any types and returns Unit
        self.env.bind(
            "print".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::TypeVar("T".to_string())], // Simplified - real impl would be variadic
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        // println function
        self.env.bind(
            "println".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Unit)),
                constraints: Vec::new(),
            },
        );

        // len function
        self.env.bind(
            "len".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Int)),
                constraints: Vec::new(),
            },
        );

        // str function
        self.env.bind(
            "str".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::String)),
                constraints: Vec::new(),
            },
        );

        // Math functions from interpreter builtins
        self.env.bind(
            "abs".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::TypeVar("T".to_string())],
                    Box::new(Type::TypeVar("T".to_string())),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "sin".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "cos".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "sqrt".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        // Math functions from stdlib
        self.env.bind(
            "tan".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "exp".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "log".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "pow".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![
                        Type::TypeVar("T".to_string()),
                        Type::TypeVar("U".to_string()),
                    ],
                    Box::new(Type::Float),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "floor".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "ceil".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "round".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::TypeVar("T".to_string())], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "max".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![
                        Type::TypeVar("T".to_string()),
                        Type::TypeVar("T".to_string()),
                    ],
                    Box::new(Type::TypeVar("T".to_string())),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "min".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![
                        Type::TypeVar("T".to_string()),
                        Type::TypeVar("T".to_string()),
                    ],
                    Box::new(Type::TypeVar("T".to_string())),
                ),
                constraints: Vec::new(),
            },
        );

        // Physics functions
        self.env.bind(
            "create_physics_world".to_string(),
            InferredType {
                ty: Type::Function(vec![], Box::new(Type::Int)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "add_rigid_body".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![
                        Type::Int,
                        Type::String,
                        Type::Float,
                        Type::Array(Box::new(Type::Float)),
                    ],
                    Box::new(Type::Int),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "physics_step".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::Int], Box::new(Type::Unit)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "get_object_position".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int],
                    Box::new(Type::Array(Box::new(Type::Float))),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "get_object_info".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int],
                    Box::new(Type::TypeVar("T".to_string())),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "get_object_mass".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Float)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "set_object_mass".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Float],
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "get_object_shape".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::Int, Type::Int], Box::new(Type::String)),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "list_objects".to_string(),
            InferredType {
                ty: Type::Function(vec![Type::Int], Box::new(Type::Array(Box::new(Type::Int)))),
                constraints: Vec::new(),
            },
        );

        // Quantum functions
        self.env.bind(
            "quantum_circuit".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int],
                    Box::new(Type::Int), // Return circuit ID as Int
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "hadamard".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "cnot".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Int], // circuit_id, control, target
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        // Parametric rotation gates
        self.env.bind(
            "rx".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Float], // circuit_id, qubit, angle
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "ry".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Float], // circuit_id, qubit, angle
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "rz".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Float], // circuit_id, qubit, angle
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "simulate_circuit".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int],     // circuit_id
                    Box::new(Type::Int), // Return result ID as Int
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "print_state".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int], // result_id
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        // Additional quantum gate functions
        // Single-qubit gates
        self.env.bind(
            "h".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "x".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "y".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "z".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "t".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "s".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        // Two-qubit gates
        self.env.bind(
            "cz".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Int], // circuit_id, qubit1, qubit2
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "swap".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Int], // circuit_id, qubit1, qubit2
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        // Three-qubit gates
        self.env.bind(
            "toffoli".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int, Type::Int, Type::Int], // circuit_id, control1, control2, target
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        // Measurement functions
        self.env.bind(
            "measure".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int, Type::Int], // circuit_id, qubit
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "measure_all".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int], // circuit_id
                    Box::new(Type::Unit),
                ),
                constraints: Vec::new(),
            },
        );

        // Additional simulation functions
        self.env.bind(
            "simulate".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int],                          // circuit_id
                    Box::new(Type::TypeVar("T".to_string())), // Simulation result
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "get_probabilities".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int],                              // circuit_id
                    Box::new(Type::Array(Box::new(Type::Float))), // Array of probabilities
                ),
                constraints: Vec::new(),
            },
        );

        // Circuit info functions
        self.env.bind(
            "circuit_info".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int], // circuit_id
                    Box::new(Type::String),
                ),
                constraints: Vec::new(),
            },
        );

        self.env.bind(
            "quantum_state_info".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![Type::Int],                          // circuit_id
                    Box::new(Type::TypeVar("T".to_string())), // State info struct
                ),
                constraints: Vec::new(),
            },
        );

        // Bell state convenience function
        self.env.bind(
            "bell_state".to_string(),
            InferredType {
                ty: Type::Function(
                    vec![],              // no parameters
                    Box::new(Type::Int), // returns circuit_id
                ),
                constraints: Vec::new(),
            },
        );
    }

    pub fn fresh_type_var(&mut self) -> Type {
        let var = format!("T{}", self.next_type_var);
        self.next_type_var += 1;
        Type::TypeVar(var)
    }

    pub fn push_scope(&mut self) {
        let new_env = TypeEnv::with_parent(std::mem::take(&mut self.env));
        self.env = new_env;
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.env.parent.take() {
            self.env = *parent;
        }
    }
}

/// Helper functions for type operations
impl Type {
    pub fn substitute(&self, substitutions: &HashMap<String, Type>) -> Type {
        match self {
            Type::TypeVar(name) => substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| self.clone()),
            Type::Array(element_type) => {
                Type::Array(Box::new(element_type.substitute(substitutions)))
            }
            Type::Matrix(element_type, rows, cols) => Type::Matrix(
                Box::new(element_type.substitute(substitutions)),
                *rows,
                *cols,
            ),
            Type::Function(params, return_type) => {
                let new_params = params.iter().map(|p| p.substitute(substitutions)).collect();
                let new_return = Box::new(return_type.substitute(substitutions));
                Type::Function(new_params, new_return)
            }
            Type::TypeApp(name, args) => {
                let new_args = args.iter().map(|a| a.substitute(substitutions)).collect();
                Type::TypeApp(name.clone(), new_args)
            }
            Type::Option(inner) => Type::Option(Box::new(inner.substitute(substitutions))),
            Type::Spanned(inner, span) => {
                Type::Spanned(Box::new(inner.substitute(substitutions)), span.clone())
            }
            Type::Field(inner) => Type::Field(Box::new(inner.substitute(substitutions))),
            Type::GPU(inner) => Type::GPU(Box::new(inner.substitute(substitutions))),
            Type::SIMD(inner, lanes) => {
                Type::SIMD(Box::new(inner.substitute(substitutions)), *lanes)
            }
            Type::Future(inner) => Type::Future(Box::new(inner.substitute(substitutions))),
            Type::Stream(inner) => Type::Stream(Box::new(inner.substitute(substitutions))),
            _ => self.clone(),
        }
    }

    pub fn occurs_check(&self, var_name: &str) -> bool {
        match self {
            Type::TypeVar(name) => name == var_name,
            Type::Array(element_type) => element_type.occurs_check(var_name),
            Type::Matrix(element_type, _, _) => element_type.occurs_check(var_name),
            Type::Function(params, return_type) => {
                params.iter().any(|p| p.occurs_check(var_name))
                    || return_type.occurs_check(var_name)
            }
            Type::TypeApp(_, args) => args.iter().any(|a| a.occurs_check(var_name)),
            Type::Option(inner) => inner.occurs_check(var_name),
            Type::Spanned(inner, _) => inner.occurs_check(var_name),
            Type::Field(inner) => inner.occurs_check(var_name),
            Type::GPU(inner) => inner.occurs_check(var_name),
            Type::SIMD(inner, _) => inner.occurs_check(var_name),
            Type::Future(inner) => inner.occurs_check(var_name),
            Type::Stream(inner) => inner.occurs_check(var_name),
            _ => false,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::String => "String".to_string(),
            Type::Unit => "Unit".to_string(),
            Type::Struct(name) => name.clone(),
            Type::Array(element_type) => format!("[{}]", element_type.to_string()),
            Type::Matrix(element_type, rows, cols) => match (rows, cols) {
                (Some(r), Some(c)) => format!("Matrix<{}, {}, {}>", element_type.to_string(), r, c),
                _ => format!("Matrix<{}>", element_type.to_string()),
            },
            Type::Function(params, return_type) => {
                let param_strs: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                format!("({}) -> {}", param_strs.join(", "), return_type.to_string())
            }
            Type::TypeVar(name) => name.clone(),
            Type::TypeApp(name, args) => {
                if args.is_empty() {
                    name.clone()
                } else {
                    let arg_strs: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                    format!("{}<{}>", name, arg_strs.join(", "))
                }
            }
            Type::Option(inner) => format!("Option<{}>", inner.to_string()),
            Type::Spanned(inner, _) => inner.to_string(),
            // Physics-specific types
            Type::Vector3 => "Vector3".to_string(),
            Type::Vector2 => "Vector2".to_string(),
            Type::Quaternion => "Quaternion".to_string(),
            Type::Transform => "Transform".to_string(),
            Type::RigidBody => "RigidBody".to_string(),
            Type::SoftBody => "SoftBody".to_string(),
            Type::FluidSystem => "FluidSystem".to_string(),
            Type::Particle => "Particle".to_string(),
            Type::Field(inner) => format!("Field<{}>", inner.to_string()),
            Type::ForceField => "ForceField".to_string(),
            Type::Material => "Material".to_string(),
            Type::Constraint => "Constraint".to_string(),
            Type::PhysicsWorld => "PhysicsWorld".to_string(),
            Type::Tensor(dimensions) => format!(
                "Tensor<{}>",
                dimensions
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            // GPU and SIMD types
            Type::GPU(inner) => format!("GPU<{}>", inner.to_string()),
            Type::SIMD(inner, lanes) => format!("SIMD<{}, {}>", inner.to_string(), lanes),
            // Async types
            Type::Future(inner) => format!("Future<{}>", inner.to_string()),
            Type::Stream(inner) => format!("Stream<{}>", inner.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Span, Type};

    fn create_test_span() -> Span {
        Span::new(0, 10, 1, 5)
    }

    #[test]
    fn test_type_mismatch_error() {
        let error = TypeError::TypeMismatch {
            expected: "Int".to_string(),
            found: "String".to_string(),
            line: 1,
            column: 5,
        };

        assert!(error.to_string().contains("Type mismatch"));
        assert!(error.to_string().contains("expected Int"));
        assert!(error.to_string().contains("found String"));
    }

    #[test]
    fn test_unknown_identifier_error() {
        let error = TypeError::UnknownIdentifier {
            name: "undefined_var".to_string(),
            line: 2,
            column: 10,
        };

        assert!(error.to_string().contains("Unknown identifier"));
        assert!(error.to_string().contains("undefined_var"));
    }

    #[test]
    fn test_field_not_found_error() {
        let error = TypeError::FieldNotFound {
            field: "x".to_string(),
            type_name: "Point".to_string(),
            line: 3,
            column: 15,
        };

        assert!(error.to_string().contains("Field x not found"));
        assert!(error.to_string().contains("type Point"));
    }

    #[test]
    fn test_struct_registry() {
        let mut registry = StructRegistry::new();

        let test_struct = StructDef {
            name: "Point".to_string(),
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    type_annotation: Type::Float,
                    optional: false,
                    default_value: None,
                    span: create_test_span(),
                },
                StructField {
                    name: "y".to_string(),
                    type_annotation: Type::Float,
                    optional: true,
                    default_value: None,
                    span: create_test_span(),
                },
            ],
            span: create_test_span(),
        };

        registry.register(test_struct);

        assert!(registry.get("Point").is_some());
        assert!(registry.get("NonExistent").is_none());
        assert!(registry.has_field("Point", "x"));
        assert!(registry.has_field("Point", "y"));
        assert!(!registry.has_field("Point", "z"));
        assert_eq!(registry.get_field_type("Point", "x"), Some(&Type::Float));
        assert!(!registry.is_field_optional("Point", "x"));
        assert!(registry.is_field_optional("Point", "y"));
    }

    #[test]
    fn test_typeclass_registry() {
        let mut registry = TypeclassRegistry::new();

        let test_typeclass = TypeclassDef {
            name: "Addable".to_string(),
            type_param: "T".to_string(),
            methods: vec![TypeclassMethod {
                name: "add".to_string(),
                type_signature: Type::Function(
                    vec![
                        Type::TypeVar("T".to_string()),
                        Type::TypeVar("T".to_string()),
                    ],
                    Box::new(Type::TypeVar("T".to_string())),
                ),
                span: create_test_span(),
            }],
            span: create_test_span(),
        };

        registry.register_typeclass(test_typeclass);
        assert!(registry.get_typeclass("Addable").is_some());
        assert!(registry.get_typeclass("NonExistent").is_none());
    }

    #[test]
    fn test_type_context_new() {
        let context = TypeContext::new();
        assert_eq!(context.next_type_var, 0);
        // Check that built-ins are registered
        assert!(context.typeclasses.get_typeclass("Addable").is_some());
    }

    #[test]
    fn test_type_context_fresh_type_var() {
        let mut context = TypeContext::new();

        let var1 = context.fresh_type_var();
        let var2 = context.fresh_type_var();

        assert_ne!(var1, var2);

        if let Type::TypeVar(name1) = var1 {
            if let Type::TypeVar(name2) = var2 {
                assert_ne!(name1, name2);
                assert!(name1.starts_with("T"));
                assert!(name2.starts_with("T"));
            } else {
                panic!("Expected TypeVar");
            }
        } else {
            panic!("Expected TypeVar");
        }
    }

    #[test]
    fn test_type_display() {
        assert_eq!(Type::Int.to_string(), "Int");
        assert_eq!(Type::Float.to_string(), "Float");
        assert_eq!(Type::Bool.to_string(), "Bool");
        assert_eq!(Type::String.to_string(), "String");
        assert_eq!(Type::Unit.to_string(), "Unit");

        let array_type = Type::Array(Box::new(Type::Int));
        assert_eq!(array_type.to_string(), "[Int]");

        let matrix_type = Type::Matrix(Box::new(Type::Float), Some(3), Some(3));
        assert_eq!(matrix_type.to_string(), "Matrix<Float, 3, 3>");

        let func_type = Type::Function(vec![Type::Int, Type::Float], Box::new(Type::Bool));
        assert_eq!(func_type.to_string(), "(Int, Float) -> Bool");

        let option_type = Type::Option(Box::new(Type::String));
        assert_eq!(option_type.to_string(), "Option<String>");
    }

    #[test]
    fn test_inferred_type_creation() {
        let inferred = InferredType {
            ty: Type::Int,
            constraints: vec![],
        };

        assert_eq!(inferred.ty, Type::Int);
        assert!(inferred.constraints.is_empty());
    }

    #[test]
    fn test_type_result_alias() {
        let success: TypeResult<Type> = Ok(Type::Int);
        let failure: TypeResult<Type> = Err(TypeError::UnknownIdentifier {
            name: "test".to_string(),
            line: 1,
            column: 1,
        });

        assert!(success.is_ok());
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_equality() {
        let error1 = TypeError::TypeMismatch {
            expected: "Int".to_string(),
            found: "String".to_string(),
            line: 1,
            column: 1,
        };

        let error2 = TypeError::TypeMismatch {
            expected: "Int".to_string(),
            found: "String".to_string(),
            line: 1,
            column: 1,
        };

        let error3 = TypeError::TypeMismatch {
            expected: "Float".to_string(),
            found: "String".to_string(),
            line: 1,
            column: 1,
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            TypeError::TypeMismatch {
                expected: "Int".to_string(),
                found: "String".to_string(),
                line: 1,
                column: 1,
            },
            TypeError::UnknownIdentifier {
                name: "x".to_string(),
                line: 1,
                column: 1,
            },
            TypeError::UnknownType {
                name: "CustomType".to_string(),
                line: 1,
                column: 1,
            },
            TypeError::FieldNotFound {
                field: "x".to_string(),
                type_name: "Point".to_string(),
                line: 1,
                column: 1,
            },
            TypeError::NotCallable {
                type_name: "Int".to_string(),
                line: 1,
                column: 1,
            },
            TypeError::WrongArgumentCount {
                expected: 2,
                found: 1,
                line: 1,
                column: 1,
            },
        ];

        assert_eq!(errors.len(), 6);
    }
}
