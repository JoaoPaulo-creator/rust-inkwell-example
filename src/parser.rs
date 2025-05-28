use crate::ast::*;
use crate::error::CompileError;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn eat(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1
        }
    }
    fn expect(&mut self, expected: Token) -> Result<(), CompileError> {
        if *self.peek() == expected {
            self.eat();
            Ok(())
        } else {
            Err(CompileError::Parse(format!(
                "Expected {:?}, found {:?}",
                expected,
                self.peek()
            )))
        }
    }

    /// Top‐level entry: parse a whole program.
    pub fn parse_program(&mut self) -> Result<Program, CompileError> {
        let mut funcs = Vec::new();
        let mut stmts = Vec::new();

        while *self.peek() != Token::EOF {
            if *self.peek() == Token::Fn {
                funcs.push(self.parse_function()?);
            } else {
                let stmt = self.parse_statement()?;
                stmts.push(stmt);
                if *self.peek() == Token::Semicolon {
                    self.eat();
                }
            }
        }

        Ok(Program {
            functions: funcs,
            statements: stmts,
        })
    }

    /// Parse `fn name(arg1, arg2, …) { … }`
    fn parse_function(&mut self) -> Result<Function, CompileError> {
        self.expect(Token::Fn)?;
        let name = match self.peek() {
            Token::Ident(n) => n.clone(),
            _ => return Err(CompileError::Parse("Expected function name".into())),
        };
        self.eat();
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        if *self.peek() != Token::RParen {
            loop {
                if let Token::Ident(n) = self.peek() {
                    params.push(n.clone());
                    self.eat();
                } else {
                    return Err(CompileError::Parse("Expected parameter name".into()));
                }
                if *self.peek() == Token::Comma {
                    self.eat();
                    continue;
                }
                break;
            }
        }
        self.expect(Token::RParen)?;
        let body = self.parse_block()?;
        Ok(Function { name, params, body })
    }

    /// Parse a `{ stmt; stmt; … }` block
    fn parse_block(&mut self) -> Result<Vec<Statement>, CompileError> {
        self.expect(Token::LBrace)?;
        let mut v = Vec::new();
        while *self.peek() != Token::RBrace {
            let stmt = self.parse_statement()?;
            self.expect(Token::Semicolon)?;
            v.push(stmt);
        }
        self.expect(Token::RBrace)?;
        Ok(v)
    }

    /// Parse any single statement (var, assign, if, while, return, print, expr‐stmt).
    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        match self.peek() {
            Token::Var => {
                self.eat();
                let name = if let Token::Ident(n) = self.peek() {
                    n.clone()
                } else {
                    return Err(CompileError::Parse("Expected var name".into()));
                };
                self.eat();
                self.expect(Token::Eq)?;
                let expr = self.parse_expr()?;
                Ok(Statement::VarDecl { name, expr })
            }
            Token::Let => {
                self.eat();
                let name = if let Token::Ident(n) = self.peek() {
                    n.clone()
                } else {
                    return Err(CompileError::Parse("Expected let name".into()));
                };
                self.eat();
                self.expect(Token::Eq)?;
                let expr = self.parse_expr()?;
                Ok(Statement::LetDecl { name, expr })
            }
            Token::If => {
                self.eat();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;
                let then_branch = self.parse_block()?;
                let else_branch = if *self.peek() == Token::Else {
                    self.eat();
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Statement::If {
                    cond,
                    then_branch,
                    else_branch,
                })
            }
            Token::While => {
                self.eat();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;
                let body = self.parse_block()?;
                Ok(Statement::While { cond, body })
            }
            Token::Return => {
                self.eat();
                let expr = self.parse_expr()?;
                Ok(Statement::Return { expr })
            }
            Token::Print => {
                self.eat();
                let expr = self.parse_expr()?;
                Ok(Statement::Print { expr })
            }
            Token::Ident(name)
                if {
                    // could be assignment or function‐call expr
                    let lookahead = &self.tokens.get(self.pos + 1).unwrap_or(&Token::EOF);
                    matches!(lookahead, Token::Eq)
                } =>
            {
                // assignment: ident = expr
                let name = name.clone();
                self.eat();
                self.expect(Token::Eq)?;
                let expr = self.parse_expr()?;
                Ok(Statement::Assign { name, expr })
            }
            _ => {
                // fallback: bare expression statement
                let expr = self.parse_expr()?;
                Ok(Statement::ExprStmt(expr))
            }
        }
    }

    /// Parse expressions with correct precedence:
    /// equality -> comparison -> addition -> term -> factor -> primary
    fn parse_expr(&mut self) -> Result<Expr, CompileError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_comparison()?;
        while matches!(self.peek(), Token::EqEq | Token::Ne) {
            let op = match self.peek() {
                Token::EqEq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                _ => unreachable!(),
            };
            self.eat();
            let rhs = self.parse_comparison()?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_addition()?;
        while matches!(self.peek(), Token::Lt | Token::Le | Token::Gt | Token::Ge) {
            let op = match self.peek() {
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => unreachable!(),
            };
            self.eat();
            let rhs = self.parse_addition()?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_addition(&mut self) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_term()?;
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = if *self.peek() == Token::Plus {
                BinOp::Add
            } else {
                BinOp::Sub
            };
            self.eat();
            let rhs = self.parse_term()?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_factor()?;
        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Rem,
                _ => unreachable!(),
            };
            self.eat();
            let rhs = self.parse_factor()?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self) -> Result<Expr, CompileError> {
        match self.peek() {
            Token::Plus => {
                self.eat();
                let e = self.parse_factor()?;
                Ok(Expr::Unary {
                    op: UnOp::Pos,
                    expr: Box::new(e),
                })
            }
            Token::Minus => {
                self.eat();
                let e = self.parse_factor()?;
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(e),
                })
            }
            Token::Number(n) => {
                let v = *n;
                self.eat();
                Ok(Expr::Number(v))
            }
            Token::BoolLiteral(b) => {
                let v = *b;
                self.eat();
                Ok(Expr::Bool(v))
            }
            Token::StrLiteral(s) => {
                let v = s.clone();
                self.eat();
                Ok(Expr::StrLiteral(v))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.eat();
                // function call?
                if *self.peek() == Token::LParen {
                    self.eat();
                    let mut args = Vec::new();
                    if *self.peek() != Token::RParen {
                        loop {
                            args.push(self.parse_expr()?);
                            if *self.peek() == Token::Comma {
                                self.eat();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Token::LParen => {
                self.eat();
                let e = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(e)
            }
            other => Err(CompileError::Parse(format!(
                "Unexpected token in factor: {:?}",
                other
            ))),
        }
    }
}
