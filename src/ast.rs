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
    If {
        condition: Box<Span<Expr>>,
        branch_then: Vec<Span<Expr>>,
        branch_else: Vec<Span<Expr>>,
    }
}

#[derive(Debug)]
pub enum Statement {
    Error(ParseError),
    Expr(Expr),
    Let {
        name: Span<String>,
        value: Span<Expr>,
    },
    Item(Item),
}

#[derive(Debug)]
pub struct FunctionArgument {
    pub name: Span<String>,
}

#[derive(Debug)]
pub enum Item {
    Error(ParseError),
    Function {
        name: Span<String>,
        args: Vec<Span<FunctionArgument>>,
        body: Vec<Span<Statement>>,
    },
    Class {
        name: Span<String>,
        members: Vec<Span<Item>>,
    },
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub items: Vec<Span<Item>>,
}
