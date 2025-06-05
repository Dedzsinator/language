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
        self.bindings.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup(name))
        })
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
        self.typeclasses.insert(typeclass_def.name.clone(), typeclass_def);
    }
    
    pub fn register_instance(&mut self, instance_def: InstanceDef) {
        let key = (instance_def.typeclass_name.clone(), instance_def.type_name.clone());
        self.instances.insert(key, instance_def);
    }
    
    pub fn get_typeclass(&self, name: &str) -> Option<&TypeclassDef> {
        self.typeclasses.get(name)
    }
    
    pub fn get_instance(&self, typeclass: &str, type_name: &str) -> Option<&InstanceDef> {
        self.instances.get(&(typeclass.to_string(), type_name.to_string()))
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
                    vec![Type::TypeVar("T".to_string()), Type::TypeVar("T".to_string())],
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
                body: Expression::Identifier("__builtin_add_int".to_string(), Span::new(0, 0, 0, 0)),
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
                body: Expression::Identifier("__builtin_add_float".to_string(), Span::new(0, 0, 0, 0)),
                span: Span::new(0, 0, 0, 0),
            }],
            span: Span::new(0, 0, 0, 0),
        };
        self.typeclasses.register_instance(float_addable);
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
            Type::TypeVar(name) => {
                substitutions.get(name).cloned().unwrap_or_else(|| self.clone())
            }
            Type::Array(element_type) => {
                Type::Array(Box::new(element_type.substitute(substitutions)))
            }
            Type::Matrix(element_type, rows, cols) => {
                Type::Matrix(Box::new(element_type.substitute(substitutions)), *rows, *cols)
            }
            Type::Function(params, return_type) => {
                let new_params = params.iter().map(|p| p.substitute(substitutions)).collect();
                let new_return = Box::new(return_type.substitute(substitutions));
                Type::Function(new_params, new_return)
            }
            Type::TypeApp(name, args) => {
                let new_args = args.iter().map(|a| a.substitute(substitutions)).collect();
                Type::TypeApp(name.clone(), new_args)
            }
            Type::Option(inner) => {
                Type::Option(Box::new(inner.substitute(substitutions)))
            }
            Type::Spanned(inner, span) => {
                Type::Spanned(Box::new(inner.substitute(substitutions)), span.clone())
            }
            _ => self.clone(),
        }
    }
    
    pub fn occurs_check(&self, var_name: &str) -> bool {
        match self {
            Type::TypeVar(name) => name == var_name,
            Type::Array(element_type) => element_type.occurs_check(var_name),
            Type::Matrix(element_type, _, _) => element_type.occurs_check(var_name),
            Type::Function(params, return_type) => {
                params.iter().any(|p| p.occurs_check(var_name)) || return_type.occurs_check(var_name)
            }
            Type::TypeApp(_, args) => args.iter().any(|a| a.occurs_check(var_name)),
            Type::Option(inner) => inner.occurs_check(var_name),
            Type::Spanned(inner, _) => inner.occurs_check(var_name),
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
            Type::Matrix(element_type, rows, cols) => {
                match (rows, cols) {
                    (Some(r), Some(c)) => format!("Matrix<{}, {}, {}>", element_type.to_string(), r, c),
                    _ => format!("Matrix<{}>", element_type.to_string()),
                }
            }
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
        }
    }
}
