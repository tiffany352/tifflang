extern crate web_assembler as wasm;
extern crate regex;

mod span;
mod lexer;
mod ast;
mod parser;
mod typecheck;
mod compiler;

use lexer::Token;
use span::Span;
use std::rc::Rc;
use std::fs::File;
use wasm::Dump;
use std::io::Write;

const TEST: &'static str = r#"
fn foo(x: int, y: int) -> int {
    let z = x * x
    z + y * 2
}
"#;/*
class Foo {
    fn foo(x, y, z) {
        1 * 2 + 3
        if 1 {
            print("foo")
        }
        else {
            1 + 1
            2 + 3
        }
        bar(x, y)
        baz("hello")
    }

    fn bar() {
        "baz"
    }
}
"#;*/

fn main() {
    let lexer = lexer::Lexer::new();
    let tokens = lexer.lex(Rc::new(TEST.to_owned()));
    /*for span in &tokens {
        let span = span.map(|token| format!("{:?}", token));
        println!("{}", span);
    }*/
    let iter: Box<Iterator<Item=Span<Token>>> = Box::new(tokens.into_iter());
    let module = parser::parse_module("stdin", &mut iter.peekable());
    let module = typecheck::typecheck_module(module);
    println!("{:#?}", module);
    let module = compiler::compile_module(&module);
    //println!("{:#?}", module);

    let mut file = File::create("output.wasm").unwrap();
    let mut code = vec![];
    module.dump(&mut code);
    file.write(&mut code).unwrap();
}
