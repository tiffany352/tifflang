use wasm::builder::{FunctionBuilder, ModuleBuilder, CodeBuilder};
use wasm::{ValueType, FuncType, Module as WasmModule, ExportEntry, ExportKind, FunctionIndex, LocalIndex};
use ast::{Module, Expr, BinOp, Statement};
use std::collections::HashMap;

fn compile_expr(bindings: &HashMap<String, LocalIndex>, cb: CodeBuilder, expr: &Expr) -> CodeBuilder {
    match *expr {
        Expr::BinOp { ref op, ref lhs, ref rhs } => {
            let cb = compile_expr(bindings, cb, lhs.get_value());
            let cb = compile_expr(bindings, cb, rhs.get_value());
            match *op {
                BinOp::Add => cb.i32_add(),
                BinOp::Sub => cb.i32_sub(),
                BinOp::Mul => cb.i32_mul(),
                BinOp::Div => unimplemented!(),
            }
        },
        Expr::Variable(ref name) => {
            if let Some(index) = bindings.get(name) {
                cb.get_local(*index)
            }
            else {
                panic!("Undefined local")
            }
        },
        Expr::ConstInteger(value) =>  {
            cb.constant(value as i32)
        },
        Expr::If { ref condition, ref branch_then, ref branch_else } => {
            let cb = compile_expr(bindings, cb, condition.get_value());
            let cb = cb.if_();
            let mut cb = cb;
            for expr in branch_then {
                cb = compile_expr(bindings, cb, expr.get_value());
            }
            cb = cb.else_();
            for expr in branch_else {
                cb = compile_expr(bindings, cb, expr.get_value());
            }
            cb.end()
        },
        Expr::Error(ref e) => panic!(format!("{:?}", e)),
        _ => unimplemented!(),
    }
}

fn compile_statement(md: &mut ModuleBuilder, stmt: &Statement) {
    match *stmt {
        Statement::Function { ref args, ref body, .. } => {
            let ty = FuncType {
                params: args.iter().map(|_| ValueType::I32).collect(),
                ret: Some(ValueType::I32),
            };
            let f = FunctionBuilder::new(ty).code(|mut cb, params| {
                let mut bindings = HashMap::new();
                for (index, param) in params.iter().enumerate() {
                    bindings.insert(
                        args[index].get_value().name.get_value().to_owned(),
                        *param
                    );
                }

                for expr in body {
                    cb = compile_expr(&bindings, cb, expr.get_value());
                }
                cb.return_()
            }).build();
            md.new_function(f);
        },
        _ => unimplemented!(),
    }
}

pub fn compile_module(module: &Module) -> WasmModule {
    let mut md = ModuleBuilder::new();
    // function to create must be the 0th function of the module...
    for statement in &module.statements {
        compile_statement(&mut md, statement.get_value());
    }

    md.add_export(ExportEntry {
        field: "main".to_owned(),
        kind: ExportKind::Function(FunctionIndex(0)),
    });

    let module = md.build();

    module
}
