use std::collections::HashMap;
use span::Span;
use ast::{Expr, BinOp, Statement, Item, Module};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TypeError {
    ParseError,
    BinOpMismatch {
        op: BinOp,
        lhs: Box<Type>,
        rhs: Box<Type>,
    },
    FunctionArgsMismatch {
        func_args: Vec<Type>,
        given_args: Vec<Type>,
    },
    CallingNonFunction {
        given_type: Box<Type>,
    },
    ExpectedType {
        expected: Box<Type>,
        given: Box<Type>,
    },
    IfBranchMismatch {
        branch_then: Box<Type>,
        branch_else: Box<Type>,
    },
    UndefinedVariable {
        name: String,
    },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Error(TypeError),
    Integer,
    Real,
    String,
    Boolean,
    Void,

    Function {
        result: Box<Type>,
        args: Vec<Type>,
    }
}

#[derive(Debug)]
pub struct Typed<T> {
    pub value: T,
    pub type_info: Option<Type>,
}

impl<T> Typed<T> {
    pub fn new(value: T) -> Typed<T> {
        Typed {
            value: value,
            type_info: None,
        }
    }

    pub fn with_type(value: T, type_info: Type) -> Typed<T> {
        Typed {
            value: value,
            type_info: Some(type_info),
        }
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

}

fn typecheck_expr(bindings: &Bindings, expr: Typed<Expr>) -> Typed<Expr> {
    match expr.value {
        expr @ Expr::Error(_) => Typed::with_type(expr, Type::Error(TypeError::ParseError)),
        expr @ Expr::ConstInteger(_) => Typed::with_type(expr, Type::Integer),
        expr @ Expr::ConstNumber(_) => Typed::with_type(expr, Type::Real),
        expr @ Expr::ConstString(_) => Typed::with_type(expr, Type::String),
        Expr::Variable(name) => {
            if let Some(type_info) = bindings.get(&name) {
                Typed::with_type(Expr::Variable(name), type_info.clone())
            }
            else {
                let type_info = Type::Error(TypeError::UndefinedVariable {
                    name: name.clone(),
                });
                Typed::with_type(Expr::Variable(name), type_info)
            }
        },

        Expr::BinOp { op, lhs, rhs } => {
            let lhs = lhs.map(|expr| typecheck_expr(bindings, expr));
            let rhs = rhs.map(|expr| typecheck_expr(bindings, expr));
            let left_ty = lhs.get_value().type_info.clone().unwrap();
            let right_ty = rhs.get_value().type_info.clone().unwrap();
            let type_info = match (&left_ty, &right_ty) {
                (Type::Integer, Type::Integer) => Type::Integer,
                (Type::Real, Type::Real) => Type::Real,
                _ => Type::Error(TypeError::BinOpMismatch {
                    op: op.clone(),
                    lhs: Box::new(left_ty.clone()),
                    rhs: Box::new(right_ty.clone()),
                })
            };
            Typed::with_type(Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs)
            }, type_info)
        },

        Expr::Call { func, args } => {
            let func = func.map(|expr| typecheck_expr(bindings, expr));
            let args: Vec<Span<Typed<Expr>>> = args.into_iter().map(|span| span.map(|expr| typecheck_expr(bindings, expr))).collect();

            let func_ty = func.get_value().type_info.clone().unwrap();
            let args_ty = args.iter().map(|span| span.get_value().type_info.clone().unwrap()).collect();
            let type_info = match func_ty {
                Type::Function { result, args } => {
                    if args == args_ty {
                        (*result).clone()
                    }
                    else {
                        Type::Error(TypeError::FunctionArgsMismatch {
                            func_args: args,
                            given_args: args_ty,
                        })
                    }
                },
                other => Type::Error(TypeError::CallingNonFunction {
                    given_type: Box::new(other),
                })
            };

            Typed::with_type(Expr::Call {
                func: Box::new(func),
                args,
            }, type_info)
        },

        Expr::If { condition, branch_then, branch_else } => {
            let condition = condition.map(|expr| typecheck_expr(bindings, expr));
            let branch_then: Vec<Span<Typed<Expr>>> = branch_then.into_iter().map(|span| span.map(|expr| typecheck_expr(bindings, expr))).collect();
            let branch_else: Vec<Span<Typed<Expr>>> = branch_else.into_iter().map(|span| span.map(|expr| typecheck_expr(bindings, expr))).collect();

            let condition_ty = condition.get_value().type_info.clone().unwrap();
            let then_ty = branch_then.last().map(|span| span.get_value().type_info.clone().unwrap()).unwrap_or(Type::Void);
            let else_ty = branch_else.last().map(|span| span.get_value().type_info.clone().unwrap()).unwrap_or(Type::Void);
            let type_info = match (condition_ty, then_ty, else_ty) {
                (Type::Boolean, ref a, ref b) if a == b => a.clone(),
                (Type::Boolean, ref a, ref b) => Type::Error(TypeError::IfBranchMismatch {
                    branch_then: Box::new(a.clone()),
                    branch_else: Box::new(b.clone()),
                }),
                (given, _, _) => Type::Error(TypeError::ExpectedType {
                    expected: Box::new(Type::Boolean),
                    given: Box::new(given),
                }),
            };

            Typed::with_type(Expr::If {
                condition: Box::new(condition),
                branch_then: branch_then,
                branch_else: branch_else,
            }, type_info)
        },
    }
}

fn typecheck_statement(bindings: &Bindings, stmt: Statement) -> Statement {
    match stmt {
        Statement::Expr(expr) => Statement::Expr(typecheck_expr(bindings, expr)),
        Statement::Let { name, value } => Statement::Let {
            name,
            value: value.map(|expr| typecheck_expr(bindings, expr)),
        },
        stmt => stmt,
    }
}

type Bindings = HashMap<String, Type>;

pub fn typecheck_item(item: Item) -> Item {
    match item {
        Item::Function { name, args, body } => {
            let mut bindings: Bindings = HashMap::new();
            for arg in &args {
                bindings.insert(
                    arg.get_value().name.get_value().clone(),
                    arg.get_value().type_desc.get_value().clone()
                );
            }
            let mut result_body = vec![];

            for statement in body {
                let statement = statement.map(|stmt| typecheck_statement(&bindings, stmt));
                match statement.get_value() {
                    Statement::Let { ref name, ref value } => {
                        bindings.insert(
                            name.get_value().clone(),
                            value.get_value().type_info.clone().unwrap()
                        );
                    }
                    _ => (),
                }
                result_body.push(statement);
            }

            Item::Function {
                name, args,
                body: result_body,
            }
        },
        Item::Class { name, members } => Item::Class {
            name,
            members: members.into_iter().map(|span| span.map(typecheck_item)).collect(),
        },
        item => item,
    }
}

pub fn typecheck_module(module: Module) -> Module {
    Module {
        name: module.name,
        items: module.items.into_iter().map(|span| span.map(typecheck_item)).collect(),
    }
}
