use std::{fs::write, io::Write, path::PathBuf};

use bip39::Mnemonic;
use clap::Parser;
use sha2::{Digest, Sha256};

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

        data
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
    let seed = seed(input, cli.iterations, cli.long, cli.unnormalized);
    cli.write_output(seed.to_string().as_bytes());
}

fn seed(input: Vec<u8>, iterations: usize, long: bool, unnormalized: bool) -> Mnemonic {
    let input = if unnormalized {
        input
    } else {
        attempt_normalize(input)
    };
    let input = hash_iterations(&input, iterations);
    let input = if long { input.as_ref() } else { &input[..16] };
    bip39::Mnemonic::from_entropy(input).unwrap()
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

    fn normal_input() -> Vec<u8> {
        "hello world".as_bytes().to_vec()
    }

    fn abnormal_input() -> Vec<u8> {
        "Hel!lo    wo!RLD!   ".as_bytes().to_vec()
    }

    #[test]
    fn test_seed_phrase_short() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void";
        assert_eq!(seed(normal_input(), 1, false, false).to_string(), expected);
    }

    #[test]
    fn test_seed_phrase_short_abnormal() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void";
        assert_eq!(
            seed(abnormal_input(), 1, false, false).to_string(),
            expected
        );
    }

    #[test]
    fn test_long_seed_phrase() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void embark jewel mistake engine liberty innocent captain urban soda jewel dash daring";
        assert_eq!(seed(normal_input(), 1, true, false).to_string(), expected);
    }

    #[test]
    fn test_long_seed_phrase_abnormal() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void embark jewel mistake engine liberty innocent captain urban soda jewel dash daring";
        assert_eq!(seed(abnormal_input(), 1, true, false).to_string(), expected);
    }
}
