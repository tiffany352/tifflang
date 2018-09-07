use span::Span;
use lexer::Token;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        token: Span<Token>,
        expected: &'static str,
    }
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Expr {
    Error(ParseError),
    ConstInteger(i64),
    ConstNumber(f64),
    ConstString(String),
    Variable(String),
    BinOp {
        op: BinOp,
        lhs: Box<Span<Expr>>,
        rhs: Box<Span<Expr>>,
    },
    Call {
        func: Box<Span<Expr>>,
        args: Vec<Span<Expr>>,
    },
}

#[derive(Debug)]
pub struct FunctionArgument {
    pub name: Span<String>,
}

#[derive(Debug)]
pub enum Statement {
    Error(ParseError),
    Function {
        name: Span<String>,
        args: Vec<Span<FunctionArgument>>,
        body: Vec<Span<Expr>>,
    }
}
