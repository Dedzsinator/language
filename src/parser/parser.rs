use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenWithSpan};
use crate::parser::error::{ParseError, ParseResult};
use std::collections::HashMap;

pub struct Parser<'input> {
    lexer: Lexer<'input>,
    current_token: TokenWithSpan,
    peek_token: TokenWithSpan,
}

impl<'input> Parser<'input> {
    pub fn new(mut lexer: Lexer<'input>) -> ParseResult<Self> {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Ok(Self {
            lexer,
            current_token,
            peek_token,
        })
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let start_span = self.current_token.span.clone();
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        let end_span = if items.is_empty() {
            start_span.clone()
        } else {
            items.last().unwrap().span().clone()
        };

        Ok(Program {
            items,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_item(&mut self) -> ParseResult<Item> {
        match &self.current_token.token {
            Token::Struct => Ok(Item::StructDef(self.parse_struct_def()?)),
            Token::Typeclass => Ok(Item::TypeclassDef(self.parse_typeclass_def()?)),
            Token::Instance => Ok(Item::InstanceDef(self.parse_instance_def()?)),
            Token::Let => Ok(Item::LetBinding(self.parse_let_binding()?)),
            Token::Import => Ok(Item::Import(self.parse_import()?)),
            Token::At => {
                // Function with attributes
                let attributes = self.parse_attributes()?;
                self.expect(Token::Let)?;
                let mut func_def = self.parse_function_def()?;
                func_def.attributes = attributes;
                Ok(Item::FunctionDef(func_def))
            }
            _ => {
                return Err(ParseError::unexpected_token(
                    "struct, typeclass, instance, let, or import",
                    &self.current_token.token.to_string(),
                    &self.current_token.span,
                ));
            }
        }
    }

    fn parse_struct_def(&mut self) -> ParseResult<StructDef> {
        let start_span = self.current_token.span.clone();
        self.expect(Token::Struct)?;

        let name = self.expect_identifier()?;
        self.expect(Token::LeftBrace)?;

        let mut fields = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            fields.push(self.parse_struct_field()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else if !self.check(&Token::RightBrace) {
                return Err(ParseError::unexpected_token(
                    "comma or }",
                    &self.current_token.token.to_string(),
                    &self.current_token.span,
                ));
            }
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBrace)?;

        Ok(StructDef {
            name,
            fields,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_struct_field(&mut self) -> ParseResult<StructField> {
        let start_span = self.current_token.span.clone();
        let name = self.expect_identifier()?;

        let mut optional = false;
        if self.check(&Token::Question) {
            optional = true;
            self.advance();
        }

        self.expect(Token::Colon)?;
        let type_annotation = self.parse_type()?;

        let mut default_value = None;
        if self.check(&Token::Equal) {
            self.advance();
            default_value = Some(self.parse_expression()?);
        }

        let end_span = self.previous_span();

        Ok(StructField {
            name,
            type_annotation,
            optional,
            default_value,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_typeclass_def(&mut self) -> ParseResult<TypeclassDef> {
        let start_span = self.current_token.span.clone();
        self.expect(Token::Typeclass)?;

        let name = self.expect_identifier()?;
        let type_param = self.expect_identifier()?;

        self.expect(Token::LeftBrace)?;

        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_typeclass_method()?);
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBrace)?;

        Ok(TypeclassDef {
            name,
            type_param,
            methods,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_typeclass_method(&mut self) -> ParseResult<TypeclassMethod> {
        let start_span = self.current_token.span.clone();
        let name = self.expect_identifier()?;

        self.expect(Token::Colon)?;
        let type_signature = self.parse_type()?;

        let end_span = self.previous_span();

        Ok(TypeclassMethod {
            name,
            type_signature,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_instance_def(&mut self) -> ParseResult<InstanceDef> {
        let start_span = self.current_token.span.clone();
        self.expect(Token::Instance)?;

        let typeclass_name = self.expect_identifier()?;
        let type_name = self.expect_identifier()?;

        self.expect(Token::LeftBrace)?;

        let mut implementations = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            implementations.push(self.parse_method_impl()?);
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBrace)?;

        Ok(InstanceDef {
            typeclass_name,
            type_name,
            implementations,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_method_impl(&mut self) -> ParseResult<MethodImpl> {
        let start_span = self.current_token.span.clone();
        let name = self.expect_identifier()?;

        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();

        while !self.check(&Token::RightParen) && !self.is_at_end() {
            params.push(self.parse_parameter()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else if !self.check(&Token::RightParen) {
                return Err(ParseError::unexpected_token(
                    "comma or )",
                    &self.current_token.token.to_string(),
                    &self.current_token.span,
                ));
            }
        }

        self.expect(Token::RightParen)?;
        self.expect(Token::Equal)?;

        let body = self.parse_expression()?;
        let end_span = body.span().clone();

        Ok(MethodImpl {
            name,
            params,
            body,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_let_binding(&mut self) -> ParseResult<LetBinding> {
        let start_span = self.current_token.span.clone();
        self.expect(Token::Let)?;

        let name = self.expect_identifier()?;

        let mut type_annotation = None;
        if self.check(&Token::Colon) {
            self.advance();
            type_annotation = Some(self.parse_type()?);
        }

        self.expect(Token::Equal)?;
        let value = self.parse_expression()?;
        let end_span = value.span().clone();

        Ok(LetBinding {
            name,
            type_annotation,
            value,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_function_def(&mut self) -> ParseResult<FunctionDef> {
        let start_span = self.current_token.span.clone();
        let name = self.expect_identifier()?;

        self.expect(Token::Equal)?;

        // Parse lambda: (params) => body
        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();

        while !self.check(&Token::RightParen) && !self.is_at_end() {
            params.push(self.parse_parameter()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else if !self.check(&Token::RightParen) {
                return Err(ParseError::unexpected_token(
                    "comma or )",
                    &self.current_token.token.to_string(),
                    &self.current_token.span,
                ));
            }
        }

        self.expect(Token::RightParen)?;

        let mut return_type = None;
        if self.check(&Token::ThinArrow) {
            self.advance();
            return_type = Some(self.parse_type()?);
        }

        // Accept both => and -> for function body
        if self.check(&Token::Arrow) {
            self.advance();
        } else if self.check(&Token::ThinArrow) {
            self.advance();
        } else {
            return Err(ParseError::unexpected_token(
                "=> or ->",
                &self.current_token.token.to_string(),
                &self.current_token.span,
            ));
        }
        let body = self.parse_expression()?;
        let end_span = body.span().clone();

        Ok(FunctionDef {
            name,
            params,
            return_type,
            body,
            attributes: Vec::new(),
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_parameter(&mut self) -> ParseResult<Parameter> {
        let start_span = self.current_token.span.clone();
        let name = self.expect_identifier()?;

        self.expect(Token::Colon)?;
        let type_annotation = self.parse_type()?;
        let end_span = self.previous_span();

        Ok(Parameter {
            name,
            type_annotation,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attributes = Vec::new();

        while self.check(&Token::At) {
            let start_span = self.current_token.span.clone();
            self.advance(); // consume @

            let name = self.expect_identifier()?;
            let mut args = Vec::new();

            if self.check(&Token::LeftParen) {
                self.advance();

                while !self.check(&Token::RightParen) && !self.is_at_end() {
                    args.push(self.parse_expression()?);

                    if self.check(&Token::Comma) {
                        self.advance();
                    } else if !self.check(&Token::RightParen) {
                        return Err(ParseError::unexpected_token(
                            "comma or )",
                            &self.current_token.token.to_string(),
                            &self.current_token.span,
                        ));
                    }
                }

                self.expect(Token::RightParen)?;
            }

            let end_span = self.previous_span();

            attributes.push(Attribute {
                name,
                args,
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            });
        }

        Ok(attributes)
    }

    fn parse_import(&mut self) -> ParseResult<Import> {
        let start_span = self.current_token.span.clone();
        self.expect(Token::Import)?;

        let module_path = self.expect_identifier()?;
        let mut items = None;

        if self.check(&Token::LeftBrace) {
            self.advance();
            let mut item_list = Vec::new();

            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                item_list.push(self.expect_identifier()?);

                if self.check(&Token::Comma) {
                    self.advance();
                } else if !self.check(&Token::RightBrace) {
                    return Err(ParseError::unexpected_token(
                        "comma or }",
                        &self.current_token.token.to_string(),
                        &self.current_token.span,
                    ));
                }
            }

            self.expect(Token::RightBrace)?;
            items = Some(item_list);
        }

        let end_span = self.previous_span();

        Ok(Import {
            module_path,
            items,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    // Type parsing
    fn parse_type(&mut self) -> ParseResult<Type> {
        match &self.current_token.token {
            Token::IntType => {
                self.advance();
                Ok(Type::Int)
            }
            Token::FloatType => {
                self.advance();
                Ok(Type::Float)
            }
            Token::BoolType => {
                self.advance();
                Ok(Type::Bool)
            }
            Token::StringType => {
                self.advance();
                Ok(Type::String)
            }
            Token::UnitType => {
                self.advance();
                Ok(Type::Unit)
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check for generic type application
                if self.check(&Token::Less) {
                    self.advance();
                    let mut type_args = Vec::new();

                    while !self.check(&Token::Greater) && !self.is_at_end() {
                        type_args.push(self.parse_type()?);

                        if self.check(&Token::Comma) {
                            self.advance();
                        } else if !self.check(&Token::Greater) {
                            return Err(ParseError::unexpected_token(
                                "comma or >",
                                &self.current_token.token.to_string(),
                                &self.current_token.span,
                            ));
                        }
                    }

                    self.expect(Token::Greater)?;
                    Ok(Type::TypeApp(name, type_args))
                } else {
                    Ok(Type::Struct(name))
                }
            }
            Token::LeftBracket => {
                self.advance();
                let element_type = Box::new(self.parse_type()?);
                self.expect(Token::RightBracket)?;
                Ok(Type::Array(element_type))
            }
            Token::LeftParen => {
                self.advance();
                let mut param_types = Vec::new();

                while !self.check(&Token::RightParen) && !self.is_at_end() {
                    param_types.push(self.parse_type()?);

                    if self.check(&Token::Comma) {
                        self.advance();
                    } else if !self.check(&Token::RightParen) {
                        return Err(ParseError::unexpected_token(
                            "comma or )",
                            &self.current_token.token.to_string(),
                            &self.current_token.span,
                        ));
                    }
                }

                self.expect(Token::RightParen)?;
                self.expect(Token::ThinArrow)?;
                let return_type = Box::new(self.parse_type()?);

                Ok(Type::Function(param_types, return_type))
            }
            _ => Err(ParseError::unexpected_token(
                "type",
                &self.current_token.token.to_string(),
                &self.current_token.span,
            )),
        }
    }

    // Expression parsing with precedence climbing
    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseResult<Expression> {
        self.parse_logical_or()
    }
    fn parse_logical_or(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.check(&Token::OrOr) {
            let operator = BinaryOperator::Or;
            let _op_span = self.current_token.span.clone();
            self.advance();
            let right = self.parse_logical_and()?;
            let end_span = right.span().clone();
            let start_span = expr.span().clone();

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            };
        }

        Ok(expr)
    }
    fn parse_logical_and(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_equality()?;

        while self.check(&Token::AndAnd) {
            let operator = BinaryOperator::And;
            self.advance();
            let right = self.parse_equality()?;
            let end_span = right.span().clone();
            let start_span = expr.span().clone();

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while matches!(
            self.current_token.token,
            Token::EqualEqual | Token::NotEqual
        ) {
            let operator = match self.current_token.token {
                Token::EqualEqual => BinaryOperator::Eq,
                Token::NotEqual => BinaryOperator::Ne,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            let end_span = right.span().clone();
            let start_span = expr.span().clone();

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_term()?;

        while matches!(
            self.current_token.token,
            Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual
        ) {
            let operator = match self.current_token.token {
                Token::Greater => BinaryOperator::Gt,
                Token::GreaterEqual => BinaryOperator::Ge,
                Token::Less => BinaryOperator::Lt,
                Token::LessEqual => BinaryOperator::Le,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_term()?;
            let end_span = right.span().clone();
            let start_span = expr.span().clone();

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_factor()?;

        while matches!(self.current_token.token, Token::Plus | Token::Minus) {
            let operator = match self.current_token.token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_factor()?;
            let end_span = right.span().clone();
            let start_span = expr.span().clone();

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_unary()?;

        while matches!(
            self.current_token.token,
            Token::Star | Token::Slash | Token::Percent | Token::Caret
        ) {
            let operator = match self.current_token.token {
                Token::Star => BinaryOperator::Mul,
                Token::Slash => BinaryOperator::Div,
                Token::Percent => BinaryOperator::Mod,
                Token::Caret => BinaryOperator::Pow,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            let end_span = right.span().clone();
            let start_span = expr.span().clone();

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> ParseResult<Expression> {
        match &self.current_token.token {
            Token::Bang | Token::Minus => {
                let operator = match self.current_token.token {
                    Token::Bang => UnaryOperator::Not,
                    Token::Minus => UnaryOperator::Neg,
                    _ => unreachable!(),
                };
                let start_span = self.current_token.span.clone();
                self.advance();
                let operand = self.parse_unary()?;
                let end_span = operand.span().clone();

                Ok(Expression::UnaryOp {
                    operator,
                    operand: Box::new(operand),
                    span: Span::new(
                        start_span.start,
                        end_span.end,
                        start_span.line,
                        end_span.column,
                    ),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.current_token.token {
                Token::LeftParen => {
                    // Always try function call first
                    self.advance();
                    let mut args = Vec::new();

                    while !self.check(&Token::RightParen) && !self.is_at_end() {
                        args.push(self.parse_expression()?);

                        if self.check(&Token::Comma) {
                            self.advance();
                        } else if !self.check(&Token::RightParen) {
                            return Err(ParseError::unexpected_token(
                                "comma or )",
                                &self.current_token.token.to_string(),
                                &self.current_token.span,
                            ));
                        }
                    }
                    let end_span = self.current_token.span.clone();
                    self.expect(Token::RightParen)?;
                    let start_span = expr.span().clone();

                    expr = Expression::FunctionCall {
                        function: Box::new(expr),
                        args,
                        span: Span::new(
                            start_span.start,
                            end_span.end,
                            start_span.line,
                            end_span.column,
                        ),
                    };
                }
                Token::Dot => {
                    // Field access                    self.advance();
                    let field = self.expect_identifier()?;
                    let end_span = self.previous_span();
                    let start_span = expr.span().clone();

                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field,
                        span: Span::new(
                            start_span.start,
                            end_span.end,
                            start_span.line,
                            end_span.column,
                        ),
                    };
                }
                Token::QuestionQuestion => {
                    // Optional access with fallback
                    self.advance();
                    let fallback = self.parse_unary()?;
                    let end_span = fallback.span().clone(); // Extract field name from previous field access
                    if let Expression::FieldAccess {
                        ref object,
                        ref field,
                        ..
                    } = expr
                    {
                        let start_span = expr.span().clone();
                        expr = Expression::OptionalAccess {
                            object: object.clone(),
                            field: field.clone(),
                            fallback: Box::new(fallback),
                            span: Span::new(
                                start_span.start,
                                end_span.end,
                                start_span.line,
                                end_span.column,
                            ),
                        };
                    } else {
                        return Err(ParseError::invalid_syntax(
                            "?? operator can only be used after field access",
                            &self.current_token.span,
                        ));
                    }
                }
                Token::LeftBracket => {
                    // Array/matrix indexing                    self.advance();
                    let index = self.parse_expression()?;
                    let end_span = self.current_token.span.clone();
                    self.expect(Token::RightBracket)?;
                    let start_span = expr.span().clone();

                    expr = Expression::FunctionCall {
                        function: Box::new(Expression::Identifier(
                            "index".to_string(),
                            self.current_token.span.clone(),
                        )),
                        args: vec![expr.clone(), index],
                        span: Span::new(
                            start_span.start,
                            end_span.end,
                            start_span.line,
                            end_span.column,
                        ),
                    };
                }
                Token::LeftBrace => {
                    // Struct creation with brace syntax: Vector2 { x: 1, y: 2 }
                    if let Expression::Identifier(name, start_span) = expr {
                        expr = self.parse_struct_creation(name, start_span)?;
                    } else {
                        return Err(ParseError::unexpected_token(
                            "identifier",
                            "complex expression",
                            expr.span(),
                        ));
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> ParseResult<Expression> {
        match &self.current_token.token {
            Token::True => {
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Expression::BoolLiteral(true, span))
            }
            Token::False => {
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Expression::BoolLiteral(false, span))
            }
            Token::IntLiteral(value) => {
                let value = *value;
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Expression::IntLiteral(value, span))
            }
            Token::FloatLiteral(value) => {
                let value = *value;
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Expression::FloatLiteral(value, span))
            }
            Token::StringLiteral(value) => {
                let value = value.clone();
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Expression::StringLiteral(value, span))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Expression::Identifier(name, span))
            }
            Token::LeftParen => {
                self.advance();

                // Check for lambda: (params) => body
                if self.check_lambda_start() {
                    self.parse_lambda()
                } else {
                    // Parenthesized expression
                    let expr = self.parse_expression()?;
                    self.expect(Token::RightParen)?;
                    Ok(expr)
                }
            }
            Token::LeftBracket => self.parse_array_or_matrix(),
            Token::LeftBrace => self.parse_block(),
            Token::If => self.parse_if_expression(),
            Token::Match => self.parse_match_expression(),
            Token::Let => self.parse_let_expression(),
            Token::Parallel => self.parse_parallel(),
            Token::Spawn => self.parse_spawn(),
            Token::Wait => self.parse_wait(),
            Token::Gpu => self.parse_gpu_directive(),
            _ => Err(ParseError::unexpected_token(
                "expression",
                &self.current_token.token.to_string(),
                &self.current_token.span,
            )),
        }
    }

    fn parse_struct_creation(&mut self, name: String, start_span: Span) -> ParseResult<Expression> {
        let mut fields = HashMap::new();

        if self.check(&Token::LeftParen) {
            // Positional syntax: Vector2(x = 1, y = 2)
            self.advance();

            while !self.check(&Token::RightParen) && !self.is_at_end() {
                let field_name = self.expect_identifier()?;
                self.expect(Token::Equal)?;
                let field_value = self.parse_expression()?;
                fields.insert(field_name, field_value);

                if self.check(&Token::Comma) {
                    self.advance();
                } else if !self.check(&Token::RightParen) {
                    return Err(ParseError::unexpected_token(
                        "comma or )",
                        &self.current_token.token.to_string(),
                        &self.current_token.span,
                    ));
                }
            }

            let end_span = self.current_token.span.clone();
            self.expect(Token::RightParen)?;

            Ok(Expression::StructCreation {
                name,
                fields,
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            })
        } else {
            // Brace syntax: Vector2 { x: 1, y: 2 }
            self.expect(Token::LeftBrace)?;

            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                let field_name = self.expect_identifier()?;
                self.expect(Token::Colon)?;
                let field_value = self.parse_expression()?;
                fields.insert(field_name, field_value);

                if self.check(&Token::Comma) {
                    self.advance();
                } else if !self.check(&Token::RightBrace) {
                    return Err(ParseError::unexpected_token(
                        "comma or }",
                        &self.current_token.token.to_string(),
                        &self.current_token.span,
                    ));
                }
            }

            let end_span = self.current_token.span.clone();
            self.expect(Token::RightBrace)?;

            Ok(Expression::StructCreation {
                name,
                fields,
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            })
        }
    }

    fn check_lambda_start(&self) -> bool {
        // Look for parameter pattern: identifier : type
        if let Token::Identifier(_) = self.current_token.token {
            if let Token::Colon = self.peek_token.token {
                return true;
            }
        }
        false
    }

    fn parse_lambda(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        let mut params = Vec::new();

        while !self.check(&Token::RightParen) && !self.is_at_end() {
            params.push(self.parse_parameter()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else if !self.check(&Token::RightParen) {
                return Err(ParseError::unexpected_token(
                    "comma or )",
                    &self.current_token.token.to_string(),
                    &self.current_token.span,
                ));
            }
        }

        self.expect(Token::RightParen)?;

        // Accept both => and -> for lambda body
        if self.check(&Token::Arrow) {
            self.advance();
        } else if self.check(&Token::ThinArrow) {
            self.advance();
        } else {
            return Err(ParseError::unexpected_token(
                "=> or ->",
                &self.current_token.token.to_string(),
                &self.current_token.span,
            ));
        }

        let body = self.parse_expression()?;
        let end_span = body.span().clone();

        Ok(Expression::Lambda {
            params,
            body: Box::new(body),
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_array_or_matrix(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume [

        if self.check(&Token::RightBracket) {
            // Empty array
            let end_span = self.current_token.span.clone();
            self.advance();
            return Ok(Expression::ArrayLiteral(
                Vec::new(),
                Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            ));
        }

        let first_element = self.parse_expression()?;

        // Check for comprehension: [expr | var in range]
        if self.check(&Token::Pipe) {
            return self.parse_comprehension(first_element, start_span);
        }

        let mut elements = vec![first_element];

        while self.check(&Token::Comma) {
            self.advance();
            if self.check(&Token::RightBracket) {
                break; // Trailing comma
            }
            elements.push(self.parse_expression()?);
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBracket)?;

        // Check if this is a matrix (array of arrays)
        let is_matrix = elements
            .iter()
            .all(|e| matches!(e, Expression::ArrayLiteral(_, _)));

        if is_matrix {
            let rows: Vec<Vec<Expression>> = elements
                .into_iter()
                .map(|e| {
                    if let Expression::ArrayLiteral(row, _) = e {
                        row
                    } else {
                        unreachable!()
                    }
                })
                .collect();

            Ok(Expression::MatrixLiteral(
                rows,
                Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            ))
        } else {
            Ok(Expression::ArrayLiteral(
                elements,
                Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            ))
        }
    }

    fn parse_comprehension(
        &mut self,
        element: Expression,
        start_span: Span,
    ) -> ParseResult<Expression> {
        self.advance(); // consume |

        let mut generators = Vec::new();

        loop {
            let variable = self.expect_identifier()?;
            self.expect(Token::In)?;
            let iterable = self.parse_expression()?;

            let mut condition = None;
            if self.check(&Token::If) {
                self.advance();
                condition = Some(self.parse_expression()?);
            }

            generators.push(Generator {
                variable,
                iterable,
                condition,
                span: self.previous_span(),
            });

            if self.check(&Token::Pipe) {
                self.advance();
                if self.check(&Token::RightBracket) {
                    break;
                }
            } else {
                break;
            }
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBracket)?;

        // Check if this is a matrix comprehension (nested)
        if generators.len() > 1 {
            Ok(Expression::MatrixComprehension {
                element: Box::new(element),
                generators,
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            })
        } else {
            Ok(Expression::MatrixComprehension {
                element: Box::new(element),
                generators,
                span: Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    end_span.column,
                ),
            })
        }
    }

    fn parse_block(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume {

        let mut statements = Vec::new();
        let mut result = None;

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Let) {
                statements.push(Statement::LetBinding(self.parse_let_binding()?));
            } else {
                let expr = self.parse_expression()?;

                // Check if this is the last expression (result)
                if self.check(&Token::RightBrace) {
                    result = Some(Box::new(expr));
                } else {
                    statements.push(Statement::Expression(expr));
                }
            }

            // Optional semicolon
            if self.check(&Token::Semicolon) {
                self.advance();
            }
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBrace)?;

        Ok(Expression::Block {
            statements,
            result,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_if_expression(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume if

        let condition = self.parse_expression()?;
        let then_branch = self.parse_expression()?;

        let mut else_branch = None;
        if self.check(&Token::Else) {
            self.advance();
            else_branch = Some(Box::new(self.parse_expression()?));
        }

        let end_span = else_branch
            .as_ref()
            .map(|e| e.span().clone())
            .unwrap_or_else(|| then_branch.span().clone());

        Ok(Expression::IfExpression {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_match_expression(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume match

        let expression = self.parse_expression()?;
        self.expect(Token::LeftBrace)?;

        let mut arms = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBrace)?;

        Ok(Expression::Match {
            expression: Box::new(expression),
            arms,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_match_arm(&mut self) -> ParseResult<MatchArm> {
        let start_span = self.current_token.span.clone();
        let pattern = self.parse_pattern()?;

        let mut guard = None;
        if self.check(&Token::If) {
            self.advance();
            guard = Some(self.parse_expression()?);
        }

        self.expect(Token::Arrow)?;
        let body = self.parse_expression()?;
        let end_span = body.span().clone();

        // Optional comma
        if self.check(&Token::Comma) {
            self.advance();
        }

        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match &self.current_token.token {
            Token::Underscore => {
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Pattern::Wildcard(span))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                let span = self.current_token.span.clone();
                self.advance();

                match name.as_str() {
                    "Some" => {
                        self.expect(Token::LeftParen)?;
                        let inner_pattern = self.parse_pattern()?;
                        let end_span = self.current_token.span.clone();
                        self.expect(Token::RightParen)?;

                        Ok(Pattern::Some(
                            Box::new(inner_pattern),
                            Span::new(span.start, end_span.end, span.line, end_span.column),
                        ))
                    }
                    "None" => Ok(Pattern::None(span)),
                    _ => {
                        // Check for struct pattern
                        if self.check(&Token::LeftBrace) {
                            self.advance();
                            let mut fields = HashMap::new();

                            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                                let field_name = self.expect_identifier()?;
                                self.expect(Token::Colon)?;
                                let field_pattern = self.parse_pattern()?;
                                fields.insert(field_name, field_pattern);

                                if self.check(&Token::Comma) {
                                    self.advance();
                                } else if !self.check(&Token::RightBrace) {
                                    return Err(ParseError::unexpected_token(
                                        "comma or }",
                                        &self.current_token.token.to_string(),
                                        &self.current_token.span,
                                    ));
                                }
                            }

                            let end_span = self.current_token.span.clone();
                            self.expect(Token::RightBrace)?;

                            Ok(Pattern::Struct {
                                name,
                                fields,
                                span: Span::new(
                                    span.start,
                                    end_span.end,
                                    span.line,
                                    end_span.column,
                                ),
                            })
                        } else {
                            Ok(Pattern::Identifier(name, span))
                        }
                    }
                }
            }
            Token::IntLiteral(value) => {
                let value = *value;
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Pattern::IntLiteral(value, span))
            }
            Token::FloatLiteral(value) => {
                let value = *value;
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Pattern::FloatLiteral(value, span))
            }
            Token::StringLiteral(value) => {
                let value = value.clone();
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Pattern::StringLiteral(value, span))
            }
            Token::True => {
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Pattern::BoolLiteral(true, span))
            }
            Token::False => {
                let span = self.current_token.span.clone();
                self.advance();
                Ok(Pattern::BoolLiteral(false, span))
            }
            Token::LeftBracket => {
                let start_span = self.current_token.span.clone();
                self.advance();
                let mut patterns = Vec::new();

                while !self.check(&Token::RightBracket) && !self.is_at_end() {
                    patterns.push(self.parse_pattern()?);

                    if self.check(&Token::Comma) {
                        self.advance();
                    } else if !self.check(&Token::RightBracket) {
                        return Err(ParseError::unexpected_token(
                            "comma or ]",
                            &self.current_token.token.to_string(),
                            &self.current_token.span,
                        ));
                    }
                }

                let end_span = self.current_token.span.clone();
                self.expect(Token::RightBracket)?;

                Ok(Pattern::Array(
                    patterns,
                    Span::new(
                        start_span.start,
                        end_span.end,
                        start_span.line,
                        end_span.column,
                    ),
                ))
            }
            _ => Err(ParseError::unexpected_token(
                "pattern",
                &self.current_token.token.to_string(),
                &self.current_token.span,
            )),
        }
    }

    fn parse_let_expression(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        let mut bindings = Vec::new();

        // Parse multiple let bindings
        while self.check(&Token::Let) {
            bindings.push(self.parse_let_binding()?);
        }

        self.expect(Token::In)?;
        let body = self.parse_expression()?;
        let end_span = body.span().clone();

        Ok(Expression::Let {
            bindings,
            body: Box::new(body),
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_parallel(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume parallel

        self.expect(Token::LeftBrace)?;

        let mut expressions = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            expressions.push(self.parse_expression()?);

            if self.check(&Token::Semicolon) {
                self.advance();
            }
        }

        let end_span = self.current_token.span.clone();
        self.expect(Token::RightBrace)?;

        Ok(Expression::Parallel {
            expressions,
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_spawn(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume spawn

        let expression = self.parse_expression()?;
        let end_span = expression.span().clone();

        Ok(Expression::Spawn {
            expression: Box::new(expression),
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_wait(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume wait

        let expression = self.parse_expression()?;
        let end_span = expression.span().clone();

        Ok(Expression::Wait {
            expression: Box::new(expression),
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    fn parse_gpu_directive(&mut self) -> ParseResult<Expression> {
        let start_span = self.current_token.span.clone();
        self.advance(); // consume gpu

        self.expect(Token::Colon)?;
        let expression = self.parse_expression()?;
        let end_span = expression.span().clone();

        Ok(Expression::GpuDirective {
            expression: Box::new(expression),
            span: Span::new(
                start_span.start,
                end_span.end,
                start_span.line,
                end_span.column,
            ),
        })
    }

    // Helper methods for parsing
    fn is_at_end(&self) -> bool {
        matches!(self.current_token.token, Token::Eof)
    }

    fn check(&self, token: &Token) -> bool {
        &self.current_token.token == token
    }

    fn advance(&mut self) -> TokenWithSpan {
        let previous = self.current_token.clone();
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
        previous
    }
    fn expect(&mut self, expected: Token) -> ParseResult<TokenWithSpan> {
        if self.current_token.token == expected {
            Ok(self.advance())
        } else {
            Err(ParseError::unexpected_token(
                &format!("{:?}", expected),
                &format!("{:?}", self.current_token.token),
                &self.current_token.span,
            ))
        }
    }
    fn expect_identifier(&mut self) -> ParseResult<String> {
        match &self.current_token.token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError::unexpected_token(
                "identifier",
                &format!("{:?}", self.current_token.token),
                &self.current_token.span,
            )),
        }
    }

    fn previous_span(&self) -> Span {
        // Get the span from the lexer's previous position
        // Since we advanced, we need to reconstruct this
        self.current_token.span.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_struct() {
        let input = r#"
            struct Vector2 {
                x: Float,
                y: Float
            }
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::StructDef(struct_def) => {
                assert_eq!(struct_def.name, "Vector2");
                assert_eq!(struct_def.fields.len(), 2);
                assert_eq!(struct_def.fields[0].name, "x");
                assert_eq!(struct_def.fields[1].name, "y");
            }
            _ => panic!("Expected struct definition"),
        }
    }

    #[test]
    fn test_parse_optional_field() {
        let input = r#"
            struct Body {
                pos: Vector2,
                radius?: Float = 1.0
            }
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::StructDef(struct_def) => {
                assert_eq!(struct_def.fields.len(), 2);
                assert!(!struct_def.fields[0].optional);
                assert!(struct_def.fields[1].optional);
                assert!(struct_def.fields[1].default_value.is_some());
            }
            _ => panic!("Expected struct definition"),
        }
    }
}
