mod error;
mod parser;
mod processor;
mod serialiser;
mod srt;

use crate::parser::Parser;

use std::env;
use std::io::{self, Read};

use anyhow::{anyhow, Context, Result};

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => eprintln!("An error occurred:\n{}", err),
    }
}

fn run() -> Result<()> {
    let mut args = env::args();
    let command = args
        .next()
        .ok_or_else(|| anyhow!("Missing application argument."))?;
    let argument = args
        .next()
        .ok_or_else(|| anyhow!("Usage: {} <input> <output>", command))?;
    let output = args
        .next()
        .ok_or_else(|| anyhow!("Usage: {} <input> <output>", command))?;

    let data = if argument == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    } else {
        std::fs::read_to_string(argument).context("Failed to open input file.")?
    };

    let mut parser = Parser::new();

    let subs = parser.parse(&data)?;
    if subs.is_empty() {
        Err(anyhow!("You appear to have supplied an empty file."))
    } else {
        let subs = processor::process(subs)?;
        println!("Finished");
        serialiser::serialise(subs, output)?;
        Ok(())
    }
}
