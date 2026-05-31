#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Identifier(String),
    // Keywords
    Let,
    Mut,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Entity,
    Component,
    Event,
    Coroutine,
    Yield,
    Await,
    State,
    True,
    False,
    Null,
    Struct,
    // Types
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeString,
    TypeVoid,
    TypeVec2,
    TypeVec3,
    TypeVec4,
    TypeQuat,
    TypeEntity,
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Eq,
    EqEq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    AndAnd,
    Or,
    OrOr,
    Not,
    Dot,
    DotDot,
    Arrow,
    // Delimiters
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comma,
    Colon,
    Semicolon,
    // Special
    Unknown(char),
    Eof,
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unexpected character: {0}")]
    UnexpectedCharacter(char),
    #[error("Unterminated string literal")]
    UnterminatedString,
    #[error("Invalid number literal: {0}")]
    InvalidNumber(String),
}

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.position).copied()?;
        self.position += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some('/') {
            self.advance(); // consume first /
            if self.peek() == Some('/') {
                // Single-line comment
                while let Some(ch) = self.advance() {
                    if ch == '\n' {
                        break;
                    }
                }
            } else if self.peek() == Some('*') {
                // Multi-line comment
                self.advance(); // consume *
                while let Some(ch) = self.advance() {
                    if ch == '*' && self.peek() == Some('/') {
                        self.advance(); // consume /
                        break;
                    }
                }
            } else {
                // It was just a single /, push back
                self.position -= 1;
            }
        }
    }

    fn read_string(&mut self) -> Result<Token, LexerError> {
        let mut s = String::new();
        while let Some(ch) = self.advance() {
            if ch == '"' {
                return Ok(Token::String(s));
            }
            s.push(ch);
        }
        Err(LexerError::UnterminatedString)
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        let mut is_float = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                s.push(self.advance().unwrap());
            } else if ch == '.' {
                is_float = true;
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        if is_float {
            Token::Float(s.parse().unwrap_or(0.0))
        } else {
            Token::Int(s.parse().unwrap_or(0))
        }
    }

    fn read_identifier(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        match s.as_str() {
            "let" => Token::Let,
            "mut" => Token::Mut,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "return" => Token::Return,
            "break" => Token::Break,
            "entity" => Token::Entity,
            "component" => Token::Component,
            "event" => Token::Event,
            "coroutine" => Token::Coroutine,
            "yield" => Token::Yield,
            "await" => Token::Await,
            "state" => Token::State,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "struct" => Token::Struct,
            "int" => Token::TypeInt,
            "float" => Token::TypeFloat,
            "bool" => Token::TypeBool,
            "string" => Token::TypeString,
            "void" => Token::TypeVoid,
            "vec2" => Token::TypeVec2,
            "vec3" => Token::TypeVec3,
            "vec4" => Token::TypeVec4,
            "quat" => Token::TypeQuat,
            "ent" => Token::TypeEntity,
            _ => Token::Identifier(s),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            self.skip_comment();
            self.skip_whitespace();

            let Some(ch) = self.advance() else {
                tokens.push(Token::Eof);
                return Ok(tokens);
            };

            let token = match ch {
                '+' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::PlusEq
                    } else {
                        Token::Plus
                    }
                }
                '-' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::MinusEq
                    } else if self.peek() == Some('>') {
                        self.advance();
                        Token::Arrow
                    } else {
                        Token::Minus
                    }
                }
                '*' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::StarEq
                    } else {
                        Token::Star
                    }
                }
                '/' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::SlashEq
                    } else {
                        Token::Slash
                    }
                }
                '%' => Token::Percent,
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::EqEq
                    } else {
                        Token::Eq
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::Neq
                    } else {
                        Token::Not
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::Le
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::Ge
                    } else {
                        Token::Gt
                    }
                }
                '&' => {
                    if self.peek() == Some('&') {
                        self.advance();
                        Token::AndAnd
                    } else {
                        Token::And
                    }
                }
                '|' => {
                    if self.peek() == Some('|') {
                        self.advance();
                        Token::OrOr
                    } else {
                        Token::Or
                    }
                }
                '(' => Token::OpenParen,
                ')' => Token::CloseParen,
                '{' => Token::OpenBrace,
                '}' => Token::CloseBrace,
                '[' => Token::OpenBracket,
                ']' => Token::CloseBracket,
                ',' => Token::Comma,
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                '.' => {
                    if self.peek() == Some('.') {
                        self.advance();
                        Token::DotDot
                    } else {
                        Token::Dot
                    }
                }
                '"' => self.read_string()?,
                ch if ch.is_ascii_digit() => self.read_number(ch),
                ch if ch.is_alphabetic() || ch == '_' => self.read_identifier(ch),
                ch => Token::Unknown(ch),
            };
            tokens.push(token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Eof]);
    }

    #[test]
    fn test_integer_literal() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Int(42), Token::Eof]);
    }

    #[test]
    fn test_float_literal() {
        let mut lexer = Lexer::new("3.14");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Float(3.14), Token::Eof]);
    }

    #[test]
    fn test_keyword_let() {
        let mut lexer = Lexer::new("let x = 5");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Identifier("x".to_string()),
                Token::Eq,
                Token::Int(5),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello\"");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string()), Token::Eof]);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != < > <= >= += -= *= /= && || !");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::EqEq,
                Token::Neq,
                Token::Lt,
                Token::Gt,
                Token::Le,
                Token::Ge,
                Token::PlusEq,
                Token::MinusEq,
                Token::StarEq,
                Token::SlashEq,
                Token::AndAnd,
                Token::OrOr,
                Token::Not,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_comment_skipping() {
        let mut lexer = Lexer::new("// comment\n42 /* block */ 3.14");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Int(42), Token::Float(3.14), Token::Eof]);
    }

    #[test]
    fn test_entity_keyword() {
        let mut lexer = Lexer::new("entity Player { }");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Entity,
                Token::Identifier("Player".to_string()),
                Token::OpenBrace,
                Token::CloseBrace,
                Token::Eof,
            ]
        );
    }
}
