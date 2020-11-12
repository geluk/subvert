mod error;
mod parser;
mod processor;
mod serialiser;
mod srt;

use crate::parser::Parser;

use std::io::{self, Read};

use anyhow::{anyhow, Context, Result};
use clap::{Arg, App};

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("An error occurred: {}", err);
            for cause in err.chain().skip(1) {
                eprintln!("    {}", cause);
            }
        },
    }
}

fn run() -> Result<()> {
    let matches = App::new("Subvert")
        .version("0.4")
        .author("Johan Geluk <johan@geluk.io>")
        .about("Transform and clean SRT subtitles")
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .value_name("FILE")
            .help("The file to read from. If not supplied, the subtitles will be read from standard input.")
            .takes_value(true)
            .default_value("-"))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("FILE")
            .help("The file to write to. If not supplied, the subtitles will be written to standard output.")
            .takes_value(true)
            .default_value("-"))
        .arg(Arg::with_name("backup")
            .short("b")
            .long("backup")
            .value_name("FILE")
            .help("Write a backup of the original input to the specified file.")
            .takes_value(true))
        .get_matches();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();

    let data = if input == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    } else {
        std::fs::read_to_string(input).context("Failed to open input file.")?
    };

    if let Some(backup_path) = matches.value_of("backup") {
        std::fs::write(backup_path, &data)?;
    }

    let mut parser = Parser::new();

    let subs = parser.parse(&data).context(format!("Failed to parse SRT file: {}", input))?;
    if subs.is_empty() {
        return Err(anyhow!("You appear to have supplied an empty file."));
    }

    let opts = processor::ProcessOpts {
        leader_sub: None,
    };
    let subs = processor::process(subs, opts)?;
    eprintln!("Finished parsing {}", input);
    
    if output == "-" {
        let dst = io::stdout();
        serialiser::serialise(subs, dst)?;
    } else {
        let dst = std::fs::File::create(output)?;
        serialiser::serialise(subs, dst)?;
    };

    Ok(())
}
