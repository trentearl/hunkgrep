use std::path::PathBuf;

use thiserror::Error;

use atty::Stream;

use std::io::{self, Read};

use clap::{ArgAction, Parser};
use patch::Patch;

#[derive(Parser, Debug)]
#[command(name = "grephunk", version, about = "Filter hunks by line content.")]
struct Cli {
    #[arg()]
    grep: String,

    #[arg(short, long, value_name = "FILE", value_parser)]
    file: Vec<PathBuf>,

    #[arg(short = 'v', long = "invert-match", help = "Invert match (exclude matches)", action = ArgAction::SetTrue)]
    invert_match: bool,

    #[arg(short = 'i', long = "ignore-case", help = "Ignore case distinctions", action = ArgAction::SetTrue)]
    ignore_case: bool,

    #[arg(short = 'l', long = "files-with-matches", help = "Print only names of files with matches", action = ArgAction::SetTrue)]
    files_with_matches: bool,
}

pub struct Params {
    pub grep: String,

    pub invert_match: bool,
    pub ignore_case: bool,
    pub files_with_matches: bool,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Patch parsing error: {0}")]
    Patch(String),
}

pub fn parse<'a>() -> Result<(String, Vec<Patch<'a>>, Params), ParseError> {
    let cli = Cli::parse();

    let mut contents = String::new();

    if !atty::is(Stream::Stdin) {
        io::stdin().read_to_string(&mut contents)?;
    }

    for file in cli.file {
        contents.push_str(&std::fs::read_to_string(file)?);
    }

    let contents_ref: &'a str = Box::leak(contents.into_boxed_str());

    let patches =
        Patch::from_multiple(contents_ref).map_err(|e| ParseError::Patch(e.to_string()))?;

    let params = Params {
        grep: cli.grep,
        invert_match: cli.invert_match,
        ignore_case: cli.ignore_case,
        files_with_matches: cli.files_with_matches,
    };

    Ok((contents_ref.to_string(), patches, params))
}
