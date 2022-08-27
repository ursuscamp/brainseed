use std::path::PathBuf;

use clap::Parser;
use sha2::{Digest, Sha256};

// TODO: conduct normalization on the input phrase
// TODO: add tests
// TODO: add option to output raw seed instead of seed phrase
// TODO: add option to output to file instead of stdout

#[derive(clap::Parser)]
pub struct Cli {
    #[clap(
        value_parser,
        help = "Input string to use as entropy for BIP-32 seed phrase"
    )]
    input: Option<String>,

    #[clap(short, long, help = "Use an file as input instead of command line")]
    file: Option<PathBuf>,

    #[clap(short = 'n', default_value = "1000")]
    iterations: usize,

    #[clap(short, long, help = "Return as 24 word seed phrase [default: 12]")]
    long: bool,
}

impl Cli {
    fn get_input(&self) -> Vec<u8> {
        if let Some(input) = &self.input {
            return input.as_bytes().to_vec();
        } else if let Some(file) = &self.file {
            if let Ok(contents) = std::fs::read(file) {
                return contents;
            } else {
                exit_with_error("Unable to read file.");
            }
        }

        exit_with_error("No input given.");
    }
}

fn main() {
    let cli = Cli::parse();

    let input = cli.get_input();
    let input = hash_iterations(input, cli.iterations);

    let input = if cli.long {
        input.as_ref()
    } else {
        &input[..16]
    };
    let seed = bip39::Mnemonic::from_entropy(input).unwrap();
    println!("{seed}");
}

fn exit_with_error(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

fn hash_iterations(mut input: Vec<u8>, iterations: usize) -> Vec<u8> {
    for _ in 0..iterations {
        let mut hasher = Sha256::new();
        hasher.update(&input);
        input = hasher.finalize().to_vec();
    }
    input
}
