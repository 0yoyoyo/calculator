use std::io::{self, Write};
use calculator::interpret;

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
                let result = interpret(&input);
                println!("{}", result);
            }
        }
    }
}