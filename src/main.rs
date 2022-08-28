use std::{fs::write, io::Write, path::PathBuf};

use clap::Parser;
use sha2::{Digest, Sha256};

// TODO: add tests

#[derive(clap::Parser)]
pub struct Cli {
    #[clap(
        value_parser,
        help = "Input string to use as entropy for BIP-32 seed phrase"
    )]
    input: Option<String>,

    #[clap(short, long, help = "Use an file as input instead of command line")]
    file: Option<PathBuf>,

    #[clap(short = 'n', default_value = "1000000")]
    iterations: usize,

    #[clap(short, long, help = "Return as 24 word seed phrase [default: 12]")]
    long: bool,

    #[clap(short, long, help = "Do not normalize valid UTF-8 strings")]
    unnormalized: bool,

    #[clap(short, long, help = "Output to file")]
    output: Option<PathBuf>,
}

impl Cli {
    fn get_input(&self) -> Vec<u8> {
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

        if self.unnormalized {
            data
        } else {
            attempt_normalize(data)
        }
    }

    fn write_output(&self, data: &[u8]) {
        if let Some(path) = &self.output {
            if let Err(e) = write(path, data) {
                exit_with_error(&format!("Error writing output file: {e}"));
            }
        } else {
            std::io::stdout().write(data).ok();
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let input = cli.get_input();
    let input = hash_iterations(&input, cli.iterations);

    let input = if cli.long {
        input.as_ref()
    } else {
        &input[..16]
    };
    let seed = bip39::Mnemonic::from_entropy(input).unwrap();
    cli.write_output(seed.to_string().as_bytes());
}

fn exit_with_error(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

fn hash_iterations(input: &[u8], iterations: usize) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let mut input = hasher.finalize_reset();
    for _ in 1..iterations {
        hasher.update(&input);
        hasher.finalize_into_reset(&mut input);
    }
    input.to_vec()
}

/// Remove invalid characters, then remove consecutive spaces ("   " becomes " "),
/// then finally trim all whitespace from the ends of the string.
fn normalize(data: &str) -> String {
    let mut next_str = String::with_capacity(data.len());
    let start = remove_invalid_chars(data);

    let mut skip_ws = false;
    for ch in start.chars() {
        if ch == ' ' && !skip_ws {
            next_str.push(ch);
            skip_ws = true;
        } else if ch != ' ' {
            next_str.push(ch);
            skip_ws = false;
        }
    }

    next_str.trim().to_string()
}

/// Convert all ASCII characters to lowercase and remove invalid characters.
/// Valid characters are [a-z0-9 ].
fn remove_invalid_chars(data: &str) -> String {
    let mut next_str = String::with_capacity(data.len());
    let start = data.to_ascii_lowercase();

    for ch in start.chars() {
        if ('a'..'z').contains(&ch) || ('0'..'9').contains(&ch) || ch == ' ' {
            next_str.push(ch);
        }
    }

    next_str
}

/// This will attempt to normalize data. If the data is a valid UTF-8 string, then it will normalize it.
/// If it is not valid UTF-8, then it assumes the file is binary and passes it straight through.
fn attempt_normalize(data: Vec<u8>) -> Vec<u8> {
    if let Ok(string) = std::str::from_utf8(&data) {
        normalize(string).into_bytes()
    } else {
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_invalid_chars() {
        assert_eq!(
            remove_invalid_chars(" Hel!lo 1    world!  "),
            " hello 1    world  "
        );
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("  Hel!lo    1  !    WORLD!!   "), "hello 1 world");
        assert_eq!(normalize("hello    world   !"), "hello world");
    }

    #[test]
    fn test_attempt_normalize() {
        assert_eq!(
            attempt_normalize(b"  hellO  1  WoRlD!!!   ".to_vec()),
            b"hello 1 world".to_vec()
        );

        let data = include_bytes!("../test/junk.dat");
        assert_eq!(attempt_normalize(data.to_vec()), data);
    }
}
