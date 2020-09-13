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

    dbg!(tokens);
}

#[cfg(test)]
mod tests {
    use super::read_line;

    #[test]
    fn it_works() {
        read_line("12 + 123");
    }
}
