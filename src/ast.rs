use span::Span;
use lexer::Token;
use typecheck::{Typed, Type};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        token: Span<Token>,
        expected: &'static str,
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
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
        lhs: Box<Span<Typed<Expr>>>,
        rhs: Box<Span<Typed<Expr>>>,
    },
    Call {
        func: Box<Span<Typed<Expr>>>,
        args: Vec<Span<Typed<Expr>>>,
    },
    If {
        condition: Box<Span<Typed<Expr>>>,
        branch_then: Vec<Span<Typed<Expr>>>,
        branch_else: Vec<Span<Typed<Expr>>>,
    }
}

#[derive(Debug)]
pub enum Statement {
    Error(ParseError),
    Expr(Typed<Expr>),
    Let {
        name: Span<String>,
        value: Span<Typed<Expr>>,
    },
    Item(Item),
}

#[derive(Debug)]
pub struct FunctionArgument {
    pub name: Span<String>,
    pub type_desc: Span<Type>,
}

#[derive(Debug)]
pub enum Item {
    Error(ParseError),
    Function {
        name: Span<String>,
        args: Vec<Span<FunctionArgument>>,
        body: Vec<Span<Statement>>,
        result: Span<Type>,
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
