use crate::ast::{BinOp, Expr, Statement};
use crate::error::CompileError;
use crate::lexer::Token;

/// Recursive descent parser for the simple language.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // Helper: get current token.
    fn current_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    // Helper: advance to the next token.
    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    // Helper: expect a specific token and consume it, or return a parse error.
    fn expect(&mut self, expected: Token) -> Result<(), CompileError> {
        if *self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(CompileError::Parse(format!(
                "Expected {}, found {}",
                expected,
                self.current_token()
            )))
        }
    }

    /// Parse the entire program (sequence of statements).
    pub fn parse(&mut self) -> Result<Vec<Statement>, CompileError> {
        let mut statements = Vec::new();
        while !matches!(self.current_token(), Token::EOF) {
            // Skip empty lines (EOL tokens)
            if matches!(self.current_token(), Token::EOL) {
                self.advance();
                continue;
            }
            // Parse one statement
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            // After a statement, expect end-of-line or EOF
            if matches!(self.current_token(), Token::EOL) {
                self.advance(); // consume the newline
                continue;
            } else if matches!(self.current_token(), Token::EOF) {
                break;
            } else {
                return Err(CompileError::Parse(format!(
                    "Expected end-of-line or EOF after statement, found {}",
                    self.current_token()
                )));
            }
        }
        Ok(statements)
    }

    // Parse a single assignment statement: <ident> '=' <expr>
    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        // let token = self.current_token().clone();
        let name = if let Token::Ident(name) = self.current_token() {
            name.clone()
        } else {
            return Err(CompileError::Parse(format!(
                "Expected identifier at start of statement, found {}",
                self.current_token()
            )));
        };
        self.advance(); // consume the identifier
        self.expect(Token::Eq)?; // expect '=' symbol
        let expr = self.parse_expr()?;
        Ok(Statement { name, value: expr })
    }

    // Parse an expression (handles + and -).
    fn parse_expr(&mut self) -> Result<Expr, CompileError> {
        // expr := term { ('+' | '-') term }
        let mut node = self.parse_term()?;
        loop {
            match self.current_token() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Add,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Sub,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                _ => break, // no more + or - at this level
            }
        }
        Ok(node)
    }

    // Parse a term (handles * and /).
    fn parse_term(&mut self) -> Result<Expr, CompileError> {
        // term := factor { ('*' | '/') factor }
        let mut node = self.parse_factor()?;
        loop {
            match self.current_token() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_factor()?;
                    node = Expr::Binary {
                        op: BinOp::Mul,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_factor()?;
                    node = Expr::Binary {
                        op: BinOp::Div,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                _ => break, // no more * or / at this level
            }
        }
        Ok(node)
    }

    // Parse a factor (number, variable, parenthesized expr, or unary +/-).
    fn parse_factor(&mut self) -> Result<Expr, CompileError> {
        // factor := Number | Ident | '(' expr ')' | '-' factor | '+' factor
        let token = self.current_token();
        match token {
            Token::Number(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Number(val))
            }
            Token::Ident(name) => {
                let var_name = name.clone();
                self.advance();
                Ok(Expr::Variable(var_name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                if !matches!(self.current_token(), Token::RParen) {
                    return Err(CompileError::Parse(format!(
                        "Expected ')', found {}",
                        self.current_token()
                    )));
                }
                self.advance(); // consume ')'
                Ok(expr)
            }
            Token::Minus => {
                // Unary minus: -X => 0 - X
                self.advance();
                let sub_expr = self.parse_factor()?;
                Ok(Expr::Binary {
                    op: BinOp::Sub,
                    left: Box::new(Expr::Number(0)),
                    right: Box::new(sub_expr),
                })
            }
            Token::Plus => {
                // Unary plus: just skip it
                self.advance();
                self.parse_factor()
            }
            Token::EOF => Err(CompileError::Parse(
                "Unexpected end-of-file in expression".to_string(),
            )),
            Token::EOL => Err(CompileError::Parse(
                "Unexpected end-of-line in expression".to_string(),
            )),
            _ => {
                // Any other token here is invalid in an expression
                Err(CompileError::Parse(format!(
                    "Unexpected token {} in expression",
                    token
                )))
            }
        }
    }
}
