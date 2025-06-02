// src/lexer.rs

use crate::error::CompileError;

#[derive(Debug, PartialEq)]
pub enum Token {
    // Keywords
    Fn,
    Let,
    Var,
    If,
    Else,
    While,
    Return,
    Print,
    // Identifiers and literals
    Ident(String),
    Number(i64),
    StrLiteral(String),
    BoolLiteral(bool),
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    Ne,
    Eq, // =
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    // Special
    Dot,
    EOF,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn lex(input: &str) -> Result<Vec<Token>, CompileError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            // Skip whitespace
            c if c.is_whitespace() => {
                chars.next();
            }
            // Two‐char operators
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::EqEq);
                } else {
                    tokens.push(Token::Eq);
                }
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ne);
                } else {
                    return Err(CompileError::Lex("Unexpected '!'".into()));
                }
            }
            // Single‐char operators/delimiters
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '%' => {
                chars.next();
                tokens.push(Token::Percent);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '[' => {
                chars.next();
                tokens.push(Token::LBracket);
            }
            ']' => {
                chars.next();
                tokens.push(Token::RBracket);
            }
            '.' => {
                chars.next();
                tokens.push(Token::Dot);
            }
            // String literal
            '"' => {
                chars.next(); // skip opening "
                let mut s = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2 == '"' {
                        chars.next();
                        break;
                    }
                    s.push(c2);
                    chars.next();
                }
                tokens.push(Token::StrLiteral(s));
            }
            // Number literal
            c if c.is_ascii_digit() => {
                let mut val = 0i64;
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        val = val * 10 + (d as u8 - b'0') as i64;
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(val));
            }
            // Identifier or keyword or boolean
            c if is_ident_start(c) => {
                let mut ident = String::new();
                ident.push(c);
                chars.next();
                while let Some(&c2) = chars.peek() {
                    if is_ident_continue(c2) {
                        ident.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let tok = match ident.as_str() {
                    "fn" => Token::Fn,
                    "var" => Token::Var,
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "return" => Token::Return,
                    "print" => Token::Print,
                    "true" => Token::BoolLiteral(true),
                    "false" => Token::BoolLiteral(false),
                    _ => Token::Ident(ident),
                };
                tokens.push(tok);
            }
            other => {
                return Err(CompileError::Lex(format!(
                    "Unexpected character '{}'",
                    other
                )));
            }
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}
