extern crate regex;

mod span;
mod lexer;
mod ast;
mod parser;

use lexer::Token;
use span::Span;
use std::rc::Rc;

const TEST: &'static str = r#"
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
"#;

fn main() {
    let lexer = lexer::Lexer::new();
    let tokens = lexer.lex(Rc::new(TEST.to_owned()));
    /*for span in &tokens {
        let span = span.map(|token| format!("{:?}", token));
        println!("{}", span);
    }*/
    let iter: Box<Iterator<Item=Span<Token>>> = Box::new(tokens.into_iter());
    let expr = parser::parse_statement(&mut iter.peekable());
    println!("{:#?}", expr);
}
