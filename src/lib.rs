#[derive(Debug, PartialEq)]
enum TokenKind {
    Number(u64),
    Plus,
    Minus,
    Asterisk,
    Slash,
}

#[derive(Debug)]
enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
enum NodeKind {
    Number(u64),
    BinOp {
        kind: BinOpKind,
        lhs: Box<NodeKind>,
        rhs: Box<NodeKind>,
    },
}

type Tokens<'a> = std::iter::Peekable<std::slice::Iter<'a, TokenKind>>;

use NodeKind::*;
use BinOpKind::*;

struct Parser;

impl Parser {
    fn new() -> Self {
        Parser
    }

    fn num(&self, tokens: &mut Tokens) -> Box<NodeKind> {
        if let TokenKind::Number(n) = tokens.next().unwrap() {
            let node = Number(*n);
            Box::new(node)
        } else {
            unreachable!();
        }
    }

    fn mul_or_div(&self, tokens: &mut Tokens) -> Box<NodeKind> {
        let mut lhs = self.num(tokens);
        while let Some(t) = tokens.peek() {
            if **t == TokenKind::Asterisk ||
               **t == TokenKind::Slash {
                let kind = match tokens.next().unwrap() {
                    TokenKind::Asterisk => Mul,
                    TokenKind::Slash => Div,
                    _ => unreachable!(),
                };
                let rhs = self.num(tokens);
                let node = BinOp {
                    kind,
                    lhs,
                    rhs,
                };
                lhs = Box::new(node);
            } else {
                break;
            }
        }
        lhs
    }

    fn add_or_sub(&self, tokens: &mut Tokens) -> Box<NodeKind> {
        let mut lhs = self.mul_or_div(tokens);
        while let Some(t) = tokens.peek() {
            if **t == TokenKind::Plus ||
               **t == TokenKind::Minus {
                let kind = match tokens.next().unwrap() {
                    TokenKind::Plus => Add,
                    TokenKind::Minus => Sub,
                    _ => unreachable!(),
                };
                let rhs = self.mul_or_div(tokens);
                let node = BinOp {
                    kind,
                    lhs,
                    rhs,
                };
                lhs = Box::new(node)
            } else {
                break;
            }
        }
        lhs
    }

    fn parse(&self, tokens: &mut Tokens) -> Box<NodeKind> {
        let ast = self.add_or_sub(tokens);
        ast
    }
}

fn tokenize(line: &str) -> Vec<TokenKind> {
    use TokenKind::*;

    let mut tokens: Vec<TokenKind> = Vec::new();
    let mut n_tmp: Vec<u8> = Vec::new();
    let mut bytes = line.as_bytes().iter().peekable();

    while let Some(byte) = bytes.next() {
        match byte {
            b'0'..=b'9' => {
                n_tmp.push(*byte);
                while let Some(byte) = bytes.peek() {
                    if byte.is_ascii_digit() {
                        n_tmp.push(*bytes.next().unwrap());
                    } else {
                        break;
                    }
                }
                let n = std::str::from_utf8(&n_tmp).unwrap().parse().unwrap();
                tokens.push(Number(n));
                n_tmp.clear();
            },
            b'+' => { tokens.push(Plus); },
            b'-' => { tokens.push(Minus); },
            b'*' => { tokens.push(Asterisk); },
            b'/' => { tokens.push(Slash); },
            b' ' => {},
            _ => {},
        }
    }

    tokens
}

fn eval(ast: NodeKind) -> u64 {
    match ast {
        Number(n) => n,
        BinOp { kind, lhs, rhs } => {
            match kind {
                Add => eval(*lhs) + eval(*rhs),
                Sub => eval(*lhs) - eval(*rhs),
                Mul => eval(*lhs) * eval(*rhs),
                Div => eval(*lhs) / eval(*rhs),
            }
        },
    }
}

pub fn interpret(line: &str) -> u64 {
    let tokens = tokenize(line);

    //println!("{:?}", tokens);

    let mut iter = tokens.iter().peekable();
    let parser = Parser::new();
    let ast = parser.parse(&mut iter);

    //println!("{:?}", ast);

    let r = eval(*ast);

    //println!("{}", r);

    r
}
