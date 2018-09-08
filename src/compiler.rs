use wasm::builder::{FunctionBuilder, ModuleBuilder, CodeBuilder};
use wasm::{ValueType, FuncType, Module as WasmModule, ExportEntry, ExportKind, FunctionIndex, LocalIndex};
use ast::{Module, Expr, BinOp, Item, Statement};
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

fn compile_statement(bindings: &HashMap<String, LocalIndex>, cb: CodeBuilder, stmt: &Statement) -> CodeBuilder {
    match *stmt {
        Statement::Expr(ref expr) => compile_expr(&*bindings, cb, expr),
        Statement::Let { ref name, ref value } => {
            let index = bindings.get(name.get_value()).unwrap();
            let cb = compile_expr(&*bindings, cb, value.get_value());
            cb.set_local(*index)
        },
        Statement::Item(_) => unimplemented!(),
        Statement::Error(ref e) => panic!(format!("{:?}", e)),
    }
}

fn locals_statement(builder: &mut FunctionBuilder, bindings: &mut HashMap<String, LocalIndex>, stmt: &Statement) {
    match *stmt {
        Statement::Let { ref name, .. } => {
            let index = builder.new_local(ValueType::I32);
            bindings.insert(name.get_value().to_owned(), index);
        },
        _ => (),
    }
}

fn compile_item(md: &mut ModuleBuilder, stmt: &Item) {
    match *stmt {
        Item::Function { ref args, ref body, .. } => {
            let ty = FuncType {
                params: args.iter().map(|_| ValueType::I32).collect(),
                ret: Some(ValueType::I32),
            };
            let mut bindings = HashMap::new();
            let mut f = FunctionBuilder::new(ty);

            for stmt in body {
                locals_statement(&mut f, &mut bindings, stmt.get_value());
            }
            
            let f = f.code(|mut cb, params| {
                for (index, param) in params.iter().enumerate() {
                    bindings.insert(
                        args[index].get_value().name.get_value().to_owned(),
                        *param
                    );
                }

                for stmt in body {
                    cb = compile_statement(&bindings, cb, stmt.get_value());
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
    for item in &module.items {
        compile_item(&mut md, item.get_value());
    }

    md.add_export(ExportEntry {
        field: "main".to_owned(),
        kind: ExportKind::Function(FunctionIndex(0)),
    });

    let module = md.build();

    module
}
