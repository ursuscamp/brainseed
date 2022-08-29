use std::{io::Write, path::PathBuf};

use crate::util::exit_with_error;

#[derive(clap::Parser, Clone)]
pub struct Cli {
    #[clap(
        value_parser,
        help = "Input string to use as entropy for BIP-32 seed phrase"
    )]
    pub input: Option<String>,

    #[clap(short, long, help = "Use an file as input instead of command line")]
    pub file: Option<PathBuf>,

    #[clap(short = 'n', default_value = "10000000")]
    pub iterations: usize,

    #[clap(short, long, help = "Return as 24 word seed phrase [default: 12]")]
    pub long: bool,

    #[clap(short, long, help = "Do not normalize valid UTF-8 strings")]
    pub unnormalized: bool,

    #[clap(short, long, help = "Output to file")]
    pub output: Option<PathBuf>,
}

impl Cli {
    pub fn get_input(&self) -> Vec<u8> {
        let data = if let Some(input) = &self.input {
            input.as_bytes().to_vec()
        } else if let Some(file) = &self.file {
            if let Ok(contents) = std::fs::read(file) {
                contents
            } else {
                exit_with_error("Unable to read file.");
            }
        } else {
            exit_with_error("No input given.");
        };

        data
    }

    pub fn write_output(&self, data: &[u8]) {
        if let Some(path) = &self.output {
            if let Err(e) = std::fs::write(path, data) {
                exit_with_error(&format!("Error writing output file: {e}"));
            }
        } else {
            std::io::stdout().write(data).ok();
        }
    }
}
