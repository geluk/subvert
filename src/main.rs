mod error;
mod parser;
mod processor;
mod serialiser;
mod srt;

use crate::parser::Parser;

use std::io::{self, Read};

use anyhow::{anyhow, Context, Result};
use clap::Parser as ClapParser;

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("An error occurred: {}", err);
            for cause in err.chain().skip(1) {
                eprintln!("    {}", cause);
            }
        }
    }
}

#[derive(ClapParser)]
#[command(about = "Transform and clean SRT subtitles")]
struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "The file to read from. If not supplied, the subtitles will be read from standard input.",
        default_value = "-"
    )]
    input: String,
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "The file to write to. If not supplied, the subtitles will be written to standard input.",
        default_value = "-"
    )]
    output: String,
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Write a backup of the original input to the specified file."
    )]
    backup: Option<String>,
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Insert the given text into the leader subtitle."
    )]
    leader_text: Option<String>,
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let data = if cli.input == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    } else {
        std::fs::read_to_string(&cli.input)
            .context(format!("Failed to open input file: '{}'", cli.input))?
    };

    if let Some(backup_path) = cli.backup {
        std::fs::write(backup_path, &data)?;
    }

    let mut parser = Parser::new();

    let subs = parser
        .parse(&data)
        .context(format!("Failed to parse SRT file: '{}'", cli.input))?;
    if subs.is_empty() {
        return Err(anyhow!("You appear to have supplied an empty file."));
    }

    let opts = processor::ProcessOpts {
        leader_sub: cli.leader_text,
    };
    let subs = processor::process(subs, opts)?;

    if cli.output == "-" {
        let dst = io::stdout();
        serialiser::serialise(subs, dst)?;
    } else {
        let dst = std::fs::File::create(cli.output)?;
        serialiser::serialise(subs, dst)?;
    };

    Ok(())
}
