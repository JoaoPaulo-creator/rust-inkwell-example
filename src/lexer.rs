use crate::error::CompileError;

/// Tokens recognized by the lexer.
#[derive(Debug, PartialEq)]
pub enum Token {
    Ident(String), // identifier (variable name)
    Number(i64),   // numeric literal
    Plus,          // '+'
    Minus,         // '-'
    Star,          // '*'
    Slash,         // '/'
    Eq,            // '='
    LParen,        // '('
    RParen,        // ')'
    EOL,           // end-of-line (newline)
    EOF,           // end-of-file
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        match self {
            Ident(name) => write!(f, "identifier {}", name),
            Number(n) => write!(f, "number {}", n),
            Plus => write!(f, "'+'"),
            Minus => write!(f, "'-'"),
            Star => write!(f, "'*'"),
            Slash => write!(f, "'/'"),
            Eq => write!(f, "'='"),
            LParen => write!(f, "'('"),
            RParen => write!(f, "')'"),
            EOL => write!(f, "end-of-line"),
            EOF => write!(f, "end-of-file"),
        }
    }
}

/// Lexical analyzer: converts input text into a sequence of tokens.
pub fn lex(input: &str) -> Result<Vec<Token>, CompileError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut line_number = 1;
    while let Some(ch) = chars.next() {
        match ch {
            // Skip whitespace
            ' ' | '\t' | '\r' => continue,
            '\n' => {
                tokens.push(Token::EOL);
                line_number += 1;
            }
            // Numeric literal
            '0'..='9' => {
                let mut value: i64 = (ch as u8 - b'0') as i64;
                // accumulate subsequent digit characters
                while let Some(next_ch) = chars.peek() {
                    if next_ch.is_ascii_digit() {
                        let digit = (*next_ch as u8 - b'0') as i64;
                        value = value
                            .checked_mul(10)
                            .and_then(|v| v.checked_add(digit))
                            .ok_or_else(|| {
                                CompileError::Lex(format!(
                                    "Number literal too large at line {}",
                                    line_number
                                ))
                            })?;
                        chars.next(); // consume the digit
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(value));
            }
            // Identifier (variable name)
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut name = String::new();
                name.push(ch);
                while let Some(next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || *next_ch == '_' {
                        name.push(*next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(name));
            }
            // Operators and symbols
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '=' => tokens.push(Token::Eq),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            // Any other character is unexpected
            _ => {
                return Err(CompileError::Lex(format!(
                    "Unexpected character '{}' at line {}",
                    ch, line_number
                )));
            }
        }
    }
    // End-of-file marker
    tokens.push(Token::EOF);
    Ok(tokens)
}
