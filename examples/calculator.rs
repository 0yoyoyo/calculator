use std::io::{self, Write};
use calculator::interpret;

const JIT_MODE: bool = true;

fn main() {
    loop {
        let mut output = io::stdout();
        output.write(b"> ").unwrap();
        output.flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "quit" => break,
            _ => {
                match interpret(&input, JIT_MODE) {
                    Ok(result) => println!("{}", result),
                    Err(_) => {},
                }
            }
        }
    }
}
