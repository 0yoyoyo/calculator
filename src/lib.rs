extern crate libc;
use libc::{c_void, c_int, size_t, PROT_READ, PROT_WRITE, PROT_EXEC};
use std::alloc::{alloc, Layout};

extern "C" {
    fn mprotect(addr: *const c_void, len: size_t, prot: c_int) -> c_int;
}

type Tokens<'a> = std::iter::Peekable<std::slice::Iter<'a, TokenKind>>;

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

use NodeKind::*;
use BinOpKind::*;

pub fn interpret(line: &str, use_jit: bool) -> Result<u8, ()> {
    let tokens = tokenize(line)?;

    //println!("{:?}", tokens);

    let mut iter = tokens.iter().peekable();
    let parser = Parser::new();
    let ast = parser.parse(&mut iter)?;

    //println!("{:?}", ast);

    let r = if use_jit {
        unsafe {
            let mut compiler = Compiler::new();

            //println!("0x{:>016x}", compiler.p_start as u64);

            compiler.gen_code(*ast);
            let code: fn() -> u8 = std::mem::transmute(compiler.p_start);

            // run generated code!
            code()

            // TODO: Code area protection should be recovered?
        }
    } else {
        eval(*ast)
    };

    //println!("{}", r);

    Ok(r)
}

fn tokenize(line: &str) -> Result<Vec<TokenKind>, ()> {
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
                let n = match std::str::from_utf8(&n_tmp).unwrap().parse() {
                    Ok(n) => n,
                    Err(_) => {
                        eprintln!("Too big number!");
                        return Err(());
                    },
                };
                tokens.push(Number(n));
                n_tmp.clear();
            },
            b'+' => { tokens.push(Plus); },
            b'-' => { tokens.push(Minus); },
            b'*' => { tokens.push(Asterisk); },
            b'/' => { tokens.push(Slash); },
            b' ' => {},
            _ => {
                eprintln!("Unexpected characters!");
                return Err(());
            },
        }
    }

    if tokens.is_empty() {
        // No error message.
        Err(())
    } else {
        Ok(tokens)
    }
}

struct Parser;

impl Parser {
    fn new() -> Self {
        Parser
    }

    fn num(&self, tokens: &mut Tokens) -> Result<Box<NodeKind>, ()> {
        if let TokenKind::Number(n) = tokens.next().unwrap() {
            let node = Number(*n);
            Ok(Box::new(node))
        } else {
            eprintln!("Invalid expression!");
            Err(())
        }
    }

    fn mul_or_div(&self, tokens: &mut Tokens) -> Result<Box<NodeKind>, ()> {
        let mut lhs = self.num(tokens)?;
        while let Some(t) = tokens.peek() {
            if **t == TokenKind::Asterisk ||
               **t == TokenKind::Slash {
                let kind = match tokens.next().unwrap() {
                    TokenKind::Asterisk => Mul,
                    TokenKind::Slash => Div,
                    _ => unreachable!(),
                };
                let rhs = self.num(tokens)?;
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
        Ok(lhs)
    }

    fn add_or_sub(&self, tokens: &mut Tokens) -> Result<Box<NodeKind>, ()> {
        let mut lhs = self.mul_or_div(tokens)?;
        while let Some(t) = tokens.peek() {
            if **t == TokenKind::Plus ||
               **t == TokenKind::Minus {
                let kind = match tokens.next().unwrap() {
                    TokenKind::Plus => Add,
                    TokenKind::Minus => Sub,
                    _ => unreachable!(),
                };
                let rhs = self.mul_or_div(tokens)?;
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
        Ok(lhs)
    }

    fn parse(&self, tokens: &mut Tokens) -> Result<Box<NodeKind>, ()> {
        let ast = self.add_or_sub(tokens)?;
        Ok(ast)
    }
}

struct Compiler {
    p_start: *mut u8,
    p_current: *mut u8,
}

const CODE_AREA_SIZE: usize = 1024;
const PAGE_SIZE: usize = 4096;

impl Compiler {
    unsafe fn new() -> Self {
        let layout = Layout::from_size_align(CODE_AREA_SIZE, PAGE_SIZE).unwrap();
        let p_start = alloc(layout);
        let r = mprotect(p_start as *const c_void, CODE_AREA_SIZE, PROT_READ|PROT_WRITE|PROT_EXEC);
        assert!(r == 0);
        Compiler {
            p_start,
            p_current: p_start,
        }
    }

    unsafe fn push_code(&mut self, code: &[u8]) {
        for b in code.iter() {
            std::ptr::write(self.p_current, *b);
            self.p_current = (self.p_current as u64 + 1) as *mut u8;
        }
    }

    unsafe fn gen_code_ast(&mut self, ast: NodeKind) {
        match ast {
            Number(n) => {
                self.push_code(&[0x6a, n]); // push {}
            },
            BinOp { kind, lhs, rhs } => {
                self.gen_code_ast(*rhs);
                self.gen_code_ast(*lhs);
                self.push_code(&[0x58]); // pop rax
                self.push_code(&[0x5f]); // pop rdi
                match kind {
                    Add => {
                        self.push_code(&[0x48, 0x01, 0xf8]); // add rax, rdi
                    },
                    Sub => {
                        self.push_code(&[0x48, 0x29, 0xf8]); // sud rax, rdi
                    },
                    Mul => {
                        self.push_code(&[0x48, 0x0f, 0xaf, 0xc7]); // imul rax, rdi
                    },
                    Div => {
                        self.push_code(&[0x48, 0x99]); // cqo
                        self.push_code(&[0x48, 0xf7, 0xff]); // idiv rdi
                    },
                }
                self.push_code(&[0x50]); // push rax
            },
        }
    }

    unsafe fn gen_code(&mut self, ast: NodeKind) {
        self.gen_code_ast(ast);
        self.push_code(&[0x58]); // pop rax
        self.push_code(&[0xc3]); // ret
    }
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
