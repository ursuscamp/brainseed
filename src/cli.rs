use std::{io::Write, path::PathBuf};

use crate::{generator::Generator, util::exit_with_error};

#[derive(clap::Parser, Clone)]
pub struct Cli {
    #[clap(
        short = 'n',
        default_value = "10000000",
        help = "Number of times to hash the passphrase"
    )]
    pub iterations: usize,

    #[clap(subcommand)]
    pub action: Action,
}

impl Cli {
    pub fn exec(&self) {
        let input = self.get_input();
        match &self.action {
            Action::Seed { long, output } => self.seed(input, *long, output),
            Action::Sign => self.sign(),
        }
    }

    fn seed(&self, input: Vec<u8>, long: bool, output: &Option<PathBuf>) {
        let mut gen = Generator {
            input,
            iterations: self.iterations,
            long,
        };
        let seed = gen.seed();
        self.write_output(output, seed.to_string().as_bytes());
    }

    fn sign(&self) {}

    fn get_input(&self) -> Vec<u8> {
        if let Ok((true, pass)) = self.get_password() {
            pass.as_bytes().to_vec()
        } else {
            exit_with_error("Prompt does not match.");
        }
    }

    fn write_output(&self, output: &Option<PathBuf>, data: &[u8]) {
        if let Some(path) = output {
            if let Err(e) = std::fs::write(path, data) {
                exit_with_error(&format!("Error writing output file: {e}"));
            }
        } else {
            std::io::stdout().write(data).ok();
        }
    }

    fn get_password(&self) -> std::io::Result<(bool, String)> {
        let pass = rpassword::prompt_password("Entropy phrase: ")?;
        let confirm = rpassword::prompt_password("Confirm: ")?;
        Ok((pass == confirm, pass))
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Action {
    /// Generate a mnemonic seed phrase.
    Seed {
        #[clap(short, long, help = "Return a 24 word seed phrase [default: 12]")]
        long: bool,

        #[clap(short, long, help = "Output to file")]
        output: Option<PathBuf>,
    },

    /// Sign a bitcoin transaction file.
    Sign,
}
