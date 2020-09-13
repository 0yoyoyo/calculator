#[derive(Debug)]
enum TokenKind {
    Number(u64),
    Plus,
    Minus,
    Asterisk,
    Slash,
}

pub fn read_line(line: &str) {
    use TokenKind::*;

    let mut tokens: Vec<TokenKind> = Vec::new();
    let mut n_tmp: Vec<u8> = Vec::new();
    let bytes = line.as_bytes().iter().peekable();

    for byte in bytes {
        match byte {
            b'0'..=b'9' => {
                n_tmp.push(*byte);
                let n = std::str::from_utf8(&n_tmp).unwrap().parse().unwrap();
                tokens.push(Number(n));
                n_tmp.clear();
            },
            b'+' => { tokens.push(Plus); },
            b'-' => { tokens.push(Minus); },
            b'*' => { tokens.push(Asterisk); },
            b'/' => { tokens.push(Slash); },
            b' ' => { println!("Space"); },
            _ => { println!("Others"); },
        }
    }

    dbg!(tokens);
}

#[cfg(test)]
mod tests {
    use super::read_line;

    #[test]
    fn it_works() {
        read_line("1 + 1");
    }
}
