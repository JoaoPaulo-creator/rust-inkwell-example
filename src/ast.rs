/// AST node for an expression.
#[derive(Debug)]
pub enum Expr {
    Number(i64),      // literal number
    Variable(String), // use of a variable
    Binary {
        // binary operation (e.g., left op right)
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// Supported binary operators.
#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// AST node for a statement (assignment).
#[derive(Debug)]
pub struct Statement {
    pub name: String, // variable name on left side
    pub value: Expr,  // expression on right side
}
