use ast::{Expr, ParseError, BinOp, Item, FunctionArgument, Module, Statement};
use lexer::Token;
use std::iter::Peekable;
use span::Span;
use typecheck::Typed;

pub type TokenIterator = Peekable<Box<Iterator<Item=Span<Token>>>>;

fn parse_if(iter: &mut TokenIterator) -> Span<Expr> {
    let condition = parse_expr(iter);

    match iter.next().unwrap().split() {
        (_span, Token::CurlyLeft) => (),
        (span, token) => return span.replace(Expr::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "{",
        }))
    };

    let mut branch_then = vec![];
    let branch_then_end;
    loop {
        branch_then.push(parse_expr(iter));

        match iter.peek().map(ToOwned::to_owned).unwrap().split() {
            (span, Token::CurlyRight) => {
                branch_then_end = span;
                iter.next();
                break;
            },
            _ => continue,
        }
    }

    match iter.peek().map(ToOwned::to_owned).unwrap().split() {
        (_span, Token::Else) => {
            iter.next();
        },
        _ => return Span::bridge(condition.peek(), branch_then_end, Expr::If {
            condition: Box::new(condition.map(Typed::new)),
            branch_then: branch_then.into_iter().map(|span| span.map(Typed::new)).collect(),
            branch_else: vec![],
        }),
    }

    match iter.next().unwrap().split() {
        (_span, Token::CurlyLeft) => (),
        (span, token) => return span.replace(Expr::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "{",
        }))
    };

    let mut branch_else = vec![];
    let branch_else_end;
    loop {
        branch_else.push(parse_expr(iter));

        match iter.peek().map(ToOwned::to_owned).unwrap().split() {
            (span, Token::CurlyRight) => {
                branch_else_end = span;
                iter.next();
                break;
            },
            _ => continue,
        }
    }

    Span::bridge(condition.peek(), branch_else_end, Expr::If {
        condition: Box::new(condition.map(Typed::new)),
        branch_then: branch_then.into_iter().map(|span| span.map(Typed::new)).collect(),
        branch_else: branch_else.into_iter().map(|span| span.map(Typed::new)).collect(),
    })
}

fn parse_const(iter: &mut TokenIterator) -> Span<Expr> {
    let (span, token) = iter.next().unwrap().split();
    match token {
        Token::If => parse_if(iter),
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
                func: Box::new(lhs.map(Typed::new)),
                args: args.into_iter().map(|span| span.map(Typed::new)).collect(),
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
                lhs: Box::new(lhs.map(Typed::new)),
                rhs: Box::new(parse_mul(iter).map(Typed::new)),
            })
        },
        Token::Slash => {
            iter.next();
            span.replace(Expr::BinOp {
                op: BinOp::Div,
                lhs: Box::new(lhs.map(Typed::new)),
                rhs: Box::new(parse_mul(iter).map(Typed::new)),
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
                lhs: Box::new(lhs.map(Typed::new)),
                rhs: Box::new(parse_add(iter).map(Typed::new)),
            })
        },
        Token::Minus => {
            iter.next();
            span.replace(Expr::BinOp {
                op: BinOp::Sub,
                lhs: Box::new(lhs.map(Typed::new)),
                rhs: Box::new(parse_add(iter).map(Typed::new)),
            })
        }
        _ => lhs
    }
}

pub fn parse_expr(iter: &mut TokenIterator) -> Span<Expr> {
    parse_add(iter)
}

fn parse_let(iter: &mut TokenIterator) -> Span<Statement> {
    let start_span = match iter.next().unwrap().split() {
        (span, Token::Let) => span,
        (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "let",
        }))
    };

    let name = match iter.next().unwrap().split() {
        (span, Token::Ident(name)) => span.replace(name),
        (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "ident",
        }))
    };

    match iter.next().unwrap().split() {
        (_span, Token::Equals) => (),
        (span, token) => return span.replace(Statement::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "=",
        }))
    };

    let expr = parse_expr(iter);

    Span::bridge(start_span, expr.peek(), Statement::Let {
        name: name,
        value: expr.map(Typed::new),
    })
}

fn parse_statement(iter: &mut TokenIterator) -> Span<Statement> {
    match iter.peek().map(ToOwned::to_owned).unwrap().split() {
        (_span, Token::Let) => parse_let(iter),
        (_span, Token::Fn) | (_span, Token::Class) => parse_item(iter).map(Statement::Item),
        _ => parse_expr(iter).map(Typed::new).map(Statement::Expr)
    }
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

fn parse_func(iter: &mut TokenIterator) -> Span<Item> {
    let start_span = match iter.next().unwrap().split() {
        (span, Token::Fn) => span,
        (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "fn",
        }))
    };

    let name = match iter.next().unwrap().split() {
        (span, Token::Ident(ident)) => span.replace(ident),
        (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "ident",
        })),
    };

    match iter.next().unwrap().split() {
        (_span, Token::ParenLeft) => (),
        (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "(",
        }))
    };

    let mut args = vec![];
    match iter.peek().map(ToOwned::to_owned).unwrap().split() {
        (_span, Token::ParenRight) => {
            iter.next();
        },
        _ => loop {
            match parse_func_arg(iter).split() {
                (span, Ok(arg)) => args.push(span.replace(arg)),
                (span, Err(err)) => return span.replace(Item::Error(err)),
            };

            match iter.next().unwrap().split() {
                (_span, Token::Comma) => continue,
                (_span, Token::ParenRight) => break,
                (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
                    token: span.replace(token),
                    expected: ", or )",
                })),
            };
        }
    }

    match iter.next().unwrap().split() {
        (_span, Token::CurlyLeft) => (),
        (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "{",
        }))
    };

    let mut body = vec![];
    let end_span;

    loop {
        match iter.peek().map(ToOwned::to_owned).unwrap().split() {
            (span, Token::CurlyRight) => {
                iter.next();
                end_span = span;
                break;
            },
            _ => (),
        }

        body.push(parse_statement(iter));
    }

    Span::bridge(start_span, end_span, Item::Function {
        name: name,
        args: args,
        body: body,
    })
}

fn parse_class(iter: &mut TokenIterator) -> Span<Item> {
    let start_span = match iter.next().unwrap().split() {
        (span, Token::Class) => span,
        (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "class",
        }))
    };

    let name = match iter.next().unwrap().split() {
        (span, Token::Ident(ident)) => span.replace(ident),
        (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "ident",
        })),
    };

    match iter.next().unwrap().split() {
        (_span, Token::CurlyLeft) => (),
        (span, token) => return span.replace(Item::Error(ParseError::UnexpectedToken {
            token: span.replace(token),
            expected: "{",
        }))
    };

    let mut members = vec![];
    let end_span;
    loop {
        members.push(parse_item(iter));

        match iter.peek().map(ToOwned::to_owned).unwrap().split() {
            (span, Token::CurlyRight) => {
                iter.next();
                end_span = span;
                break;
            },
            _ => continue,
        };
    }

    Span::bridge(start_span, end_span, Item::Class {
        name: name,
        members: members,
    })
}

pub fn parse_item(iter: &mut TokenIterator) -> Span<Item> {
    match iter.peek().map(ToOwned::to_owned).unwrap().split() {
        (_span, Token::Fn) => parse_func(iter),
        (_span, Token::Class) => parse_class(iter),
        (span, token) => {
            iter.next();
            span.replace(Item::Error(ParseError::UnexpectedToken {
                token: span.replace(token),
                expected: "item",
            }))
        },
    }
}

pub fn parse_module(name: &str, iter: &mut TokenIterator) -> Module {
    let mut items = vec![];

    loop {
        match iter.peek().map(ToOwned::to_owned).unwrap().split() {
            (_span, Token::Eof) => break,
            _ => (),
        }

        items.push(parse_item(iter));
    }

    Module {
        name: name.to_owned(),
        items: items,
    }
}