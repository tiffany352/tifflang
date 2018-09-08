use regex::{Regex, RegexSet};
use std::rc::Rc;
use span::Span;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Token {
    Ident(String),
    Number(f64),
    Integer(i64),
    String(String),
    Class,
    Struct,
    Fn,
    If,
    Else,
    ParenLeft,
    ParenRight,
    CurlyLeft,
    CurlyRight,
    Colon,
    Arrow,
    Plus,
    Minus,
    Aster,
    Slash,
    Comma,
    Eof,
}

pub struct Lexer {
    whitespace: Regex,
    set: RegexSet,
    regexes: Vec<Regex>,
}

struct Rule {
    regex: &'static str,
    process: fn(&[Span<&str>]) -> Token,
}

const REGEXES: &[Rule] = &[
    Rule {
        regex: r"^\(",
        process: |_captures| -> Token {
            Token::ParenLeft
        }
    },
    Rule {
        regex: r"^\)",
        process: |_captures| -> Token {
            Token::ParenRight
        }
    },
    Rule {
        regex: r"^\{",
        process: |_captures| -> Token {
            Token::CurlyLeft
        }
    },
    Rule {
        regex: r"^\}",
        process: |_captures| -> Token {
            Token::CurlyRight
        }
    },
    Rule {
        regex: r"^:",
        process: |_captures| -> Token {
            Token::Colon
        }
    },
    Rule {
        regex: r"^\->",
        process: |_captures| -> Token {
            Token::Arrow
        }
    },
    Rule {
        regex: r"^\+",
        process: |_captures| -> Token {
            Token::Plus
        }
    },
    Rule {
        regex: r"^\-",
        process: |_captures| -> Token {
            Token::Minus
        }
    },
    Rule {
        regex: r"^\*",
        process: |_captures| -> Token {
            Token::Aster
        }
    },
    Rule {
        regex: r"^/",
        process: |_captures| -> Token {
            Token::Slash
        }
    },
    Rule {
        regex: r"^,",
        process: |_captures| -> Token {
            Token::Comma
        }
    },

    Rule {
        regex: r"^([a-zA-Z_][a-zA-Z0-9_]*)",
        process: |captures| -> Token {
            let ident = captures[1].get_value().to_owned();
            match ident {
                "class" => Token::Class,
                "struct" => Token::Struct,
                "fn" => Token::Fn,
                "if" => Token::If,
                "else" => Token::Else,
                ident => Token::Ident(ident.to_owned()),
            }
        }
    },
    Rule {
        regex: r"^([\-\+]?[0-9]+\.[0-9]+(?:e[0-9]+(?:\.[0-9]+)?)?)",
        process: |captures| -> Token {
            Token::Number(f64::from_str(captures[1].get_value()).unwrap())
        }
    },
    Rule {
        regex: r"^([\-\+]?[0-9]+)",
        process: |captures| -> Token {
            Token::Integer(i64::from_str(captures[1].get_value()).unwrap())
        }
    },
    Rule {
        regex: r#"^"([^"]*)""#,
        process: |captures| -> Token {
            Token::String(captures[1].get_value().to_owned().to_owned())
        }
    }
];

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            whitespace: Regex::new(r"^\s+").unwrap(),
            set: RegexSet::new(REGEXES.iter().map(|rule| rule.regex)).unwrap(),
            regexes: REGEXES.iter().map(|rule| Regex::new(rule.regex).unwrap()).collect(),
        }
    }

    pub fn lex(&self, input: Rc<String>) -> Vec<Span<Token>> {
        let input_str = &input[..];
        let mut index = 0;
        let mut tokens = vec![];
        loop {
            if index >= input_str.len() {
                break;
            }
            if let Some(capture) = self.whitespace.find(&input_str[index..]) {
                index += capture.end();
                continue;
            }
            let matches = self.set.matches(&input_str[index..]);
            let mut any_match = false;
            for i in 0..REGEXES.len() {
                if matches.matched(i) {
                    let captures = self.regexes[i].captures(&input_str[index..]).unwrap();
                    let first_capture = captures.get(0).unwrap();
                    let captures: Vec<Span<&str>> = captures.iter().map(|capture| {
                        let capture = capture.unwrap();
                        Span::new(
                            capture.as_str(),
                            index + capture.start(),
                            index + capture.end(),
                            input.clone()
                        )
                    }).collect();
                    let first = index + first_capture.start();
                    let last = index + first_capture.end();
                    index = last;
                    tokens.push(Span::new(
                        (REGEXES[i].process)(&captures[..]),
                        first, last,
                        input.clone()
                    ));
                    any_match = true;
                    break;
                }
            }
            if !any_match {
                break;
            }
        }
        tokens.push(Span::new(
            Token::Eof,
            input.len(),
            input.len(),
            input.clone()
        ));
        tokens
    }
}
