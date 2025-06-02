/// A whole program: zero or more functions, then zero or more global statements.
#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    pub statements: Vec<Statement>,
}

/// A function declaration: name, parameter list, and a body of statements.
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

/// All the statements our language supports.
#[derive(Debug)]
pub enum Statement {
    VarDecl {
        name: String,
        expr: Expr,
    },
    LetDecl {
        name: String,
        expr: Expr,
    },
    Assign {
        name: String,
        expr: Expr,
    },
    Return {
        expr: Expr,
    },
    Print {
        expr: Expr,
    },
    If {
        cond: Expr,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        cond: Expr,
        body: Vec<Statement>,
    },
    ExprStmt(Expr),
}

/// All the expression forms we support.
#[derive(Debug)]
pub enum Expr {
    Number(i64),
    Bool(bool),
    StrLiteral(String),
    Variable(String),
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Length {
        array: Box<Expr>,
    },
}

/// Unary operators: + and -.
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Pos,
    Neg,
}

/// Binary operators.
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}
