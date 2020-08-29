mod parser;
mod srt;
mod error;

use parser::Parser;
use std::env;

fn main() {
    if let Some(arg) = env::args().next() {
        let mut parser = Parser::new();
        let data = std::fs::read_to_string("./tests/00-django.srt").expect("Failed to open file!");
        let subs = parser.parse(&data);
    } else {
        println!("Missing argument.")
    }
}
