#![deny(clippy::all)]

use crate::ast::*;
use crate::lexer::{Lexer, Token};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token: {0:?} at {1}:{2}")]
    UnexpectedToken(Token, usize, usize),
    #[error("Expected {expected}, got {got:?} at {line}:{col}")]
    Expected {
        expected: String,
        got: Token,
        line: usize,
        col: usize,
    },
    #[error("Lexer error: {0}")]
    LexerError(#[from] crate::lexer::LexerError),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> Token {
        self.tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek();
        self.position += 1;
        token
    }

    fn expect(&mut self, expected: &Token) -> Result<Token, ParseError> {
        let token = self.peek();
        if std::mem::discriminant(&token) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            Err(ParseError::Expected {
                expected: format!("{:?}", expected),
                got: token,
                line: 0,
                col: 0,
            })
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        let mut statements = Vec::new();
        loop {
            match self.peek() {
                Token::Eof => break,
                _ => statements.push(self.parse_declaration()?),
            }
        }
        Ok(AstNode::Program(statements))
    }

    fn parse_declaration(&mut self) -> Result<AstNode, ParseError> {
        match self.peek() {
            Token::Entity => self.parse_entity_def(),
            Token::Component => self.parse_component_def(),
            Token::Event => self.parse_event_def(),
            Token::Fn | Token::Coroutine => self.parse_fn_def(),
            Token::Let => self.parse_var_decl(),
            _ => self.parse_statement(),
        }
    }

    fn parse_entity_def(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'entity'
        let name = self.expect(&Token::Identifier(String::new()))?;

        let name = match name {
            Token::Identifier(s) => s,
            _ => unreachable!(),
        };

        self.expect(&Token::OpenBrace)?;
        let mut components = Vec::new();
        let mut events = Vec::new();

        loop {
            match self.peek() {
                Token::CloseBrace => {
                    self.advance();
                    break;
                }
                Token::Component => {
                    self.advance();
                    let comp_name = match self.advance() {
                        Token::Identifier(s) => s,
                        t => {
                            return Err(ParseError::Expected {
                                expected: "component name".to_string(),
                                got: t,
                                line: 0,
                                col: 0,
                            })
                        }
                    };
                    let mut fields = Vec::new();
                    if self.peek() == Token::OpenBrace {
                        self.advance();
                        loop {
                            match self.peek() {
                                Token::CloseBrace => {
                                    self.advance();
                                    break;
                                }
                                _ => {
                                    let field_name = match self.advance() {
                                        Token::Identifier(s) => s,
                                        t => {
                                            return Err(ParseError::Expected {
                                                expected: "field name".to_string(),
                                                got: t,
                                                line: 0,
                                                col: 0,
                                            })
                                        }
                                    };
                                    self.expect(&Token::Colon)?;
                                    let value = self.parse_expression()?;
                                    if self.peek() == Token::Comma {
                                        self.advance();
                                    }
                                    fields.push((field_name, value));
                                }
                            }
                        }
                    }
                    components.push(ComponentRef {
                        name: comp_name,
                        fields,
                    });
                }
                Token::Identifier(_) => {
                    let name = match self.peek() {
                        Token::Identifier(ref s) if s.starts_with("on_") => {
                            self.advance();
                            match name {
                                Token::Identifier(s) => s,
                                _ => unreachable!(),
                            }
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken(
                                self.peek(),
                                0,
                                0,
                            ))
                        }
                    };
                    // Parse event handler
                    self.expect(&Token::OpenParen)?;
                    let mut params = Vec::new();
                    loop {
                        match self.peek() {
                            Token::CloseParen => break,
                            _ => {
                                let param_name = match self.advance() {
                                    Token::Identifier(s) => s,
                                    t => {
                                        return Err(ParseError::Expected {
                                            expected: "parameter name".to_string(),
                                            got: t,
                                            line: 0,
                                            col: 0,
                                        })
                                    }
                                };
                                self.expect(&Token::Colon)?;
                                let param_type = self.parse_type()?;
                                params.push(Param {
                                    name: param_name,
                                    param_type,
                                });
                                if self.peek() == Token::Comma {
                                    self.advance();
                                }
                            }
                        }
                    }
                    self.expect(&Token::CloseParen)?;
                    let body = self.parse_block()?;
                    let body_nodes = match body {
                        AstNode::Block(nodes) => nodes,
                        _ => vec![body],
                    };
                    events.push(EventHandler {
                        name,
                        params,
                        body: body_nodes,
                    });
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.peek(),
                        0,
                        0,
                    ))
                }
            }
        }

        Ok(AstNode::EntityDef {
            name,
            components,
            events,
        })
    }

    fn parse_component_def(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'component'
        let name = match self.advance() {
            Token::Identifier(s) => s,
            t => {
                return Err(ParseError::Expected {
                    expected: "component name".to_string(),
                    got: t,
                    line: 0,
                    col: 0,
                })
            }
        };
        self.expect(&Token::OpenBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        loop {
            match self.peek() {
                Token::CloseBrace => {
                    self.advance();
                    break;
                }
                Token::Fn | Token::Coroutine => {
                    methods.push(self.parse_fn_def_raw()?);
                }
                _ => {
                    let field_name = match self.advance() {
                        Token::Identifier(s) => s,
                        t => {
                            return Err(ParseError::Expected {
                                expected: "field name".to_string(),
                                got: t,
                                line: 0,
                                col: 0,
                            })
                        }
                    };
                    self.expect(&Token::Colon)?;
                    let field_type = self.parse_type()?;
                    fields.push(FieldDecl {
                        name: field_name,
                        field_type,
                        default: None,
                    });
                    if self.peek() == Token::Comma {
                        self.advance();
                    }
                }
            }
        }

        Ok(AstNode::ComponentDef {
            name,
            fields,
            methods,
        })
    }

    fn parse_event_def(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'event'
        let name = match self.advance() {
            Token::Identifier(s) => s,
            t => {
                return Err(ParseError::Expected {
                    expected: "event name".to_string(),
                    got: t,
                    line: 0,
                    col: 0,
                })
            }
        };
        self.expect(&Token::OpenParen)?;
        let mut params = Vec::new();
        loop {
            match self.peek() {
                Token::CloseParen => break,
                _ => {
                    let param_name = match self.advance() {
                        Token::Identifier(s) => s,
                        t => {
                            return Err(ParseError::Expected {
                                expected: "parameter name".to_string(),
                                got: t,
                                line: 0,
                                col: 0,
                            })
                        }
                    };
                    self.expect(&Token::Colon)?;
                    let param_type = self.parse_type()?;
                    params.push(Param {
                        name: param_name,
                        param_type,
                    });
                    if self.peek() == Token::Comma {
                        self.advance();
                    }
                }
            }
        }
        self.expect(&Token::CloseParen)?;
        self.expect(&Token::Semicolon)?;
        Ok(AstNode::EventDef { name, params })
    }

    fn parse_fn_def(&mut self) -> Result<AstNode, ParseError> {
        let is_coroutine = self.peek() == Token::Coroutine;
        if is_coroutine {
            self.advance();
        }
        self.parse_fn_def_raw_with(is_coroutine)
    }

    fn parse_fn_def_raw(&mut self) -> Result<AstNode, ParseError> {
        let is_coroutine = self.peek() == Token::Coroutine;
        if is_coroutine {
            self.advance();
        }
        self.parse_fn_def_raw_with(is_coroutine)
    }

    fn parse_fn_def_raw_with(&mut self, is_coroutine: bool) -> Result<AstNode, ParseError> {
        self.expect(&Token::Fn)?;
        let name = match self.advance() {
            Token::Identifier(s) => s,
            t => {
                return Err(ParseError::Expected {
                    expected: "function name".to_string(),
                    got: t,
                    line: 0,
                    col: 0,
                })
            }
        };
        self.expect(&Token::OpenParen)?;
        let mut params = Vec::new();
        loop {
            match self.peek() {
                Token::CloseParen => break,
                _ => {
                    let param_name = match self.advance() {
                        Token::Identifier(s) => s,
                        t => {
                            return Err(ParseError::Expected {
                                expected: "parameter name".to_string(),
                                got: t,
                                line: 0,
                                col: 0,
                            })
                        }
                    };
                    self.expect(&Token::Colon)?;
                    let param_type = self.parse_type()?;
                    params.push(Param {
                        name: param_name,
                        param_type,
                    });
                    if self.peek() == Token::Comma {
                        self.advance();
                    }
                }
            }
        }
        self.expect(&Token::CloseParen)?;
        let return_type = if self.peek() == Token::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(AstNode::FnDef {
            name,
            params,
            return_type,
            is_coroutine,
            body: Box::new(body),
        })
    }

    fn parse_var_decl(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'let'
        let mutable = self.peek() == Token::Mut;
        if mutable {
            self.advance();
        }
        let name = match self.advance() {
            Token::Identifier(s) => s,
            t => {
                return Err(ParseError::Expected {
                    expected: "variable name".to_string(),
                    got: t,
                    line: 0,
                    col: 0,
                })
            }
        };
        let var_type = if self.peek() == Token::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let initializer = self.parse_expression()?;
        self.expect(&Token::Semicolon)?;
        Ok(AstNode::VarDecl {
            name,
            mutable,
            var_type,
            initializer: Box::new(initializer),
        })
    }

    fn parse_statement(&mut self) -> Result<AstNode, ParseError> {
        match self.peek() {
            Token::If => self.parse_if_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Break => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(AstNode::Break)
            }
            Token::OpenBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expression()?;
                self.expect(&Token::Semicolon)?;
                Ok(expr)
            }
        }
    }

    fn parse_block(&mut self) -> Result<AstNode, ParseError> {
        self.expect(&Token::OpenBrace)?;
        let mut statements = Vec::new();
        loop {
            match self.peek() {
                Token::CloseBrace => {
                    self.advance();
                    break;
                }
                Token::Let => {
                    statements.push(self.parse_var_decl()?);
                }
                _ => {
                    statements.push(self.parse_statement()?);
                }
            }
        }
        Ok(AstNode::Block(statements))
    }

    fn parse_if_stmt(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.peek() == Token::Else {
            self.advance();
            if self.peek() == Token::If {
                Some(Box::new(self.parse_if_stmt()?))
            } else {
                Some(Box::new(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(AstNode::IfStmt {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_while_stmt(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(AstNode::WhileStmt {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn parse_for_stmt(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'for'
        let var = match self.advance() {
            Token::Identifier(s) => s,
            t => {
                return Err(ParseError::Expected {
                    expected: "loop variable".to_string(),
                    got: t,
                    line: 0,
                    col: 0,
                })
            }
        };
        self.expect(&Token::In)?;
        let start = self.parse_expression()?;
        self.expect(&Token::DotDot)?;
        let end = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(AstNode::ForRangeStmt {
            var,
            start: Box::new(start),
            end: Box::new(end),
            body: Box::new(body),
        })
    }

    fn parse_return_stmt(&mut self) -> Result<AstNode, ParseError> {
        self.advance(); // consume 'return'
        if self.peek() == Token::Semicolon {
            self.advance();
            Ok(AstNode::ReturnStmt { value: None })
        } else {
            let value = self.parse_expression()?;
            self.expect(&Token::Semicolon)?;
            Ok(AstNode::ReturnStmt {
                value: Some(Box::new(value)),
            })
        }
    }

    fn parse_expression(&mut self) -> Result<AstNode, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<AstNode, ParseError> {
        let left = self.parse_or_expr()?;
        match self.peek() {
            Token::Eq => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(AstNode::Assignment {
                    target: Box::new(left),
                    op: AssignOp::Assign,
                    value: Box::new(value),
                })
            }
            Token::PlusEq => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(AstNode::Assignment {
                    target: Box::new(left),
                    op: AssignOp::AddAssign,
                    value: Box::new(value),
                })
            }
            Token::MinusEq => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(AstNode::Assignment {
                    target: Box::new(left),
                    op: AssignOp::SubAssign,
                    value: Box::new(value),
                })
            }
            Token::StarEq => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(AstNode::Assignment {
                    target: Box::new(left),
                    op: AssignOp::MulAssign,
                    value: Box::new(value),
                })
            }
            Token::SlashEq => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(AstNode::Assignment {
                    target: Box::new(left),
                    op: AssignOp::DivAssign,
                    value: Box::new(value),
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_or_expr(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.peek() == Token::OrOr {
            self.advance();
            let right = self.parse_and_expr()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOpKind::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_comparison()?;
        while self.peek() == Token::AndAnd {
            self.advance();
            let right = self.parse_comparison()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOpKind::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<AstNode, ParseError> {
        let left = self.parse_addition()?;
        match self.peek() {
            Token::EqEq => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(AstNode::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOpKind::Eq,
                    right: Box::new(right),
                })
            }
            Token::Neq => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(AstNode::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOpKind::Neq,
                    right: Box::new(right),
                })
            }
            Token::Lt => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(AstNode::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOpKind::Lt,
                    right: Box::new(right),
                })
            }
            Token::Gt => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(AstNode::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOpKind::Gt,
                    right: Box::new(right),
                })
            }
            Token::Le => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(AstNode::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOpKind::Le,
                    right: Box::new(right),
                })
            }
            Token::Ge => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(AstNode::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOpKind::Ge,
                    right: Box::new(right),
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_addition(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = AstNode::BinaryOp {
                        left: Box::new(left),
                        op: BinaryOpKind::Add,
                        right: Box::new(right),
                    };
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = AstNode::BinaryOp {
                        left: Box::new(left),
                        op: BinaryOpKind::Sub,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = AstNode::BinaryOp {
                        left: Box::new(left),
                        op: BinaryOpKind::Mul,
                        right: Box::new(right),
                    };
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = AstNode::BinaryOp {
                        left: Box::new(left),
                        op: BinaryOpKind::Div,
                        right: Box::new(right),
                    };
                }
                Token::Percent => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = AstNode::BinaryOp {
                        left: Box::new(left),
                        op: BinaryOpKind::Mod,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<AstNode, ParseError> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOpKind::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOpKind::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<AstNode, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::OpenParen => {
                    self.advance();
                    let mut args = Vec::new();
                    loop {
                        match self.peek() {
                            Token::CloseParen => break,
                            _ => {
                                args.push(self.parse_expression()?);
                                if self.peek() == Token::Comma {
                                    self.advance();
                                }
                            }
                        }
                    }
                    self.expect(&Token::CloseParen)?;
                    expr = AstNode::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                Token::Dot => {
                    self.advance();
                    let member = match self.advance() {
                        Token::Identifier(s) => s,
                        t => {
                            return Err(ParseError::Expected {
                                expected: "member name".to_string(),
                                got: t,
                                line: 0,
                                col: 0,
                            })
                        }
                    };
                    expr = AstNode::MemberAccess {
                        object: Box::new(expr),
                        member,
                    };
                }
                Token::OpenBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(&Token::CloseBracket)?;
                    expr = AstNode::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<AstNode, ParseError> {
        match self.advance() {
            Token::Int(val) => Ok(AstNode::Int(val)),
            Token::Float(val) => Ok(AstNode::Float(val)),
            Token::True => Ok(AstNode::Bool(true)),
            Token::False => Ok(AstNode::Bool(false)),
            Token::Null => Ok(AstNode::Null),
            Token::String(val) => Ok(AstNode::String(val)),
            Token::Identifier(val) => Ok(AstNode::Identifier(val)),
            Token::OpenParen => {
                let expr = self.parse_expression()?;
                self.expect(&Token::CloseParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken(
                self.tokens
                    .get(self.position - 1)
                    .cloned()
                    .unwrap_or(Token::Eof),
                0,
                0,
            )),
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.advance() {
            Token::TypeInt => Ok(Type::Int),
            Token::TypeFloat => Ok(Type::Float),
            Token::TypeBool => Ok(Type::Bool),
            Token::TypeString => Ok(Type::String),
            Token::TypeVoid => Ok(Type::Void),
            Token::TypeVec2 => Ok(Type::Vec2),
            Token::TypeVec3 => Ok(Type::Vec3),
            Token::TypeVec4 => Ok(Type::Vec4),
            Token::TypeQuat => Ok(Type::Quat),
            Token::TypeEntity => Ok(Type::Entity),
            Token::Identifier(name) => Ok(Type::Named(name)),
            t => Err(ParseError::Expected {
                expected: "type name".to_string(),
                got: t,
                line: 0,
                col: 0,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_source(source: &str) -> Result<AstNode, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_empty() {
        let ast = parse_source("").unwrap();
        assert_eq!(ast, AstNode::Program(vec![]));
    }

    #[test]
    fn test_parse_var_decl() {
        let ast = parse_source("let x: int = 42;").unwrap();
        assert!(matches!(ast, AstNode::Program(_)));
    }

    #[test]
    fn test_parse_if_stmt() {
        let ast = parse_source("if x > 0 { return x; }").unwrap();
        assert!(matches!(ast, AstNode::Program(_)));
    }

    #[test]
    fn test_parse_entity() {
        let source = r#"
            entity Player {
                component Transform
                component Health { max: 100, current: 100 }
            }
        "#;
        let ast = parse_source(source).unwrap();
        assert!(matches!(ast, AstNode::Program(_)));
    }

    #[test]
    fn test_parse_fn_def() {
        let source = "fn add(a: int, b: int) -> int { return a + b; }";
        let ast = parse_source(source).unwrap();
        assert!(matches!(ast, AstNode::Program(_)));
    }

    #[test]
    fn test_parse_for_loop() {
        let source = "for i in 0..10 { log(i); }";
        let ast = parse_source(source).unwrap();
        assert!(matches!(ast, AstNode::Program(_)));
    }
}
