# Tifflang

A toy programming language that I've been making. Super WIP, might not
get far.

The compiler is written in rust and outputs directly to WebAssembly.

The language is inspired mostly by Rust, C#, and Javascript, and some
ideas that have been trying to get out of my head for a long time.

A module (a file) contains items, items are things like classes (which
contain more items inside them) or functions (which contain statements),
statements are expressions or let bindings. Things that are usually
statements like `if` are expressions, like Rust.

Example:
```rust
class SomeClass {
    fn foo(x) {
        let y = 4
        let z = x * 3
        if 1 {
            z + 2
        }
        else {
            x - 4
        }
    }
}
```
