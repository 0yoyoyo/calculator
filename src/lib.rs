extern crate libc;
use libc::{c_void, c_int, size_t, PROT_READ, PROT_WRITE, PROT_EXEC};
use std::alloc::{alloc, Layout};

extern "C" {
    fn mprotect(addr: *const c_void, len: size_t, prot: c_int) -> c_int;
}

#[derive(Debug, PartialEq)]
enum TokenKind {
    Number(u8),
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
    Number(u8),
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

fn eval(ast: NodeKind) -> u8 {
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

unsafe fn prepare_code_area() -> *mut u8 {
    let layout = Layout::from_size_align(1024, 4096).unwrap();
    let p_start = alloc(layout);
    let _ = mprotect(p_start as *const c_void, 1024, PROT_READ|PROT_WRITE|PROT_EXEC);
    p_start
}

unsafe fn push_code(current: &mut *mut u8, code: &[u8]) {
    for b in code.iter() {
        std::ptr::write(*current, *b);
        *current = (*current as u64 + 1) as *mut u8;
    }
}

unsafe fn gen_code_ast(current: &mut *mut u8, ast: NodeKind) {
    match ast {
        Number(n) => {
            push_code(current, &[0x6a, n as u8]); // push {}
        },
        BinOp { kind, lhs, rhs } => {
            gen_code_ast(current, *rhs);
            gen_code_ast(current, *lhs);
            push_code(current, &[0x58]); // pop rax
            push_code(current, &[0x5f]); // pop rdi
            match kind {
                Add => {
                    push_code(current, &[0x48, 0x01, 0xf8]); // add rax, rdi
                },
                Sub => {
                    push_code(current, &[0x48, 0x29, 0xf8]); // sud rax, rdi
                },
                Mul => {
                    push_code(current, &[0x48, 0x0f, 0xaf, 0xc7]); // imul rax, rdi
                },
                Div => {
                    push_code(current, &[0x48, 0x99]); // cqo
                    push_code(current, &[0x48, 0xf7, 0xff]); // idiv rdi
                },
            }
            push_code(current, &[0x50]); // push rax
        },
    }
}

unsafe fn gen_code(p_start: *mut u8, ast: NodeKind) {
    let mut current = p_start;
    gen_code_ast(&mut current, ast);
    push_code(&mut current, &[0x58]); // pop rax
    push_code(&mut current, &[0xc3]); // ret
}

pub fn interpret(line: &str, use_jit: bool) -> u8 {
    let tokens = tokenize(line);

    //println!("{:?}", tokens);

    let mut iter = tokens.iter().peekable();
    let parser = Parser::new();
    let ast = parser.parse(&mut iter);

    //println!("{:?}", ast);

    let r = if use_jit {
        unsafe {
            let p_start = prepare_code_area();

            //println!("0x{:>016x}", p_start as u64);

            gen_code(p_start, *ast);
            let code = std::mem::transmute::<*mut u8, fn() -> u8>(p_start);

            // run generated code!
            code()
        }
    } else {
        eval(*ast)
    };

    //println!("{}", r);

    r
}
