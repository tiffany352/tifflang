use ast::{Expr, ParseError, BinOp, Statement, FunctionArgument};
use lexer::Token;
use std::iter::Peekable;
use span::Span;

pub type TokenIterator = Peekable<Box<Iterator<Item=Span<Token>>>>;

fn parse_const(iter: &mut TokenIterator) -> Span<Expr> {
    let (span, token) = iter.next().unwrap().split();
    match token {
        Token::Ident(ident) => span.replace(Expr::Variable(ident)),
        Token::Integer(int) => span.replace(Expr::ConstInteger(int)),
        Token::Number(num) => span.replace(Expr::ConstNumber(num)),
        Token::String(string) => span.replace(Expr::ConstString(string)),
        token => span.replace(Expr::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "constant",
        }))
    }
}

fn parse_call(iter: &mut TokenIterator) -> Span<Expr> {
    let lhs = parse_const(iter);

    let (span, token) = iter.peek().map(ToOwned::to_owned).unwrap().split();
    match token {
        Token::ParenLeft => {
            iter.next();
            let mut args = vec![];
            loop {
                let expr = parse_expr(iter);
                args.push(expr);
                let (span, token) = iter.next().unwrap().split();
                match token {
                    Token::Comma => continue,
                    Token::ParenRight => break,
                    token => {
                        args.push(span.replace(Expr::Error(ParseError::UnexpectedToken {
                            token: span.replace(token),
                            expected: ", or )",
                        })));
                        break;
                    }
                }
            }
            span.replace(Expr::Call {
                func: Box::new(lhs),
                args: args,
            })
        },
        _ => lhs,
    }
}

fn parse_mul(iter: &mut TokenIterator) -> Span<Expr> {
    let lhs = parse_call(iter);

    let (span, token) = iter.peek().map(ToOwned::to_owned).unwrap().split();
    match token {
        Token::Aster => {
            iter.next();
            span.replace(Expr::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(lhs),
                rhs: Box::new(parse_mul(iter)),
            })
        },
        Token::Slash => {
            iter.next();
            span.replace(Expr::BinOp {
                op: BinOp::Div,
                lhs: Box::new(lhs),
                rhs: Box::new(parse_mul(iter)),
            })
        }
        _ => lhs
    }

}

fn parse_add(iter: &mut TokenIterator) -> Span<Expr> {
    let lhs = parse_mul(iter);

    let (span, token) = iter.peek().map(ToOwned::to_owned).unwrap().split();
    match token {
        Token::Plus => {
            iter.next();
            span.replace(Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(lhs),
                rhs: Box::new(parse_add(iter)),
            })
        },
        Token::Minus => {
            iter.next();
            span.replace(Expr::BinOp {
                op: BinOp::Sub,
                lhs: Box::new(lhs),
                rhs: Box::new(parse_add(iter)),
            })
        }
        _ => lhs
    }
}

pub fn parse_expr(iter: &mut TokenIterator) -> Span<Expr> {
    parse_add(iter)
}

fn parse_func_arg(iter: &mut TokenIterator) -> Span<Result<FunctionArgument, ParseError>> {
    let name = match iter.next().unwrap().split() {
        (span, Token::Ident(ident)) => span.replace(ident),
        (span, token) => return span.replace(Err(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "ident",
        })),
    };

    name.clone().replace(Ok(FunctionArgument {
        name: name,
    }))
}

fn parse_func(iter: &mut TokenIterator) -> Span<Statement> {
    let start_span = match iter.next().unwrap().split() {
        (span, Token::Fn) => span,
        (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "fn",
        }))
    };

    let name = match iter.next().unwrap().split() {
        (span, Token::Ident(ident)) => span.replace(ident),
        (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "ident",
        })),
    };

    match iter.next().unwrap().split() {
        (_span, Token::ParenLeft) => (),
        (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "(",
        }))
    };

    let mut args = vec![];
    loop {
        match parse_func_arg(iter).split() {
            (span, Ok(arg)) => args.push(span.replace(arg)),
            (span, Err(err)) => return span.replace(Statement::Error(err)),
        };

        match iter.next().unwrap().split() {
            (_span, Token::Comma) => continue,
            (_span, Token::ParenRight) => break,
            (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
                token: span.replace(token),
                expected: ", or )",
            })),
        };
    }

    match iter.next().unwrap().split() {
        (_span, Token::CurlyLeft) => (),
        (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "{",
        }))
    };

    let mut body = vec![];
    let end_span;

    loop {
        body.push(parse_expr(iter));

        match iter.peek().map(ToOwned::to_owned).unwrap().split() {
            (span, Token::CurlyRight) => {
                end_span = span;
                break;
            },
            _ => continue,
        }
    }

    Span::bridge(start_span, end_span, Statement::Function {
        name: name,
        args: args,
        body: body,
    })
}

pub fn parse_statement(iter: &mut TokenIterator) -> Span<Statement> {
    match iter.peek().map(ToOwned::to_owned).unwrap().split() {
        (_span, Token::Fn) => parse_func(iter),
        (span, token) => span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "statement",
        })),
    }
}