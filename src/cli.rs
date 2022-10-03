use std::{io::Write, path::PathBuf};

use bdk::{
    bitcoin::{
        consensus::{deserialize, serialize},
        psbt::PartiallySignedTransaction,
        Network,
    },
    database::MemoryDatabase,
    keys::{bip39::Mnemonic, DerivableKey, ExtendedKey},
    miniscript::miniscript,
    template::Bip84,
    KeychainKind, Wallet,
};

use crate::{generator::Generator, util::exit_with_error};

#[derive(clap::Parser, Clone)]
pub struct Cli {
    #[clap(
        short = 'n',
        default_value = "10000000",
        help = "Number of times to hash the passphrase"
    )]
    pub iterations: usize,

    #[clap(short, long, help = "Return a 24 word seed phrase [default: 12]")]
    long: bool,

    #[clap(subcommand)]
    pub action: Action,
}

impl Cli {
    pub fn exec(&self) {
        let input = self.get_input();
        let seed = self.seed(input);
        match &self.action {
            Action::Seed { output } => self.write_output(output, seed.to_string().as_bytes()),
            Action::Sign { input, output } => self.sign(input, output, seed),
            Action::Descriptor => self.show_descriptor(seed),
        }
    }

    fn seed(&self, input: Vec<u8>) -> Mnemonic {
        let mut gen = Generator {
            input,
            iterations: self.iterations,
            long: self.long,
        };
        gen.seed()
    }

    fn sign(&self, input: &PathBuf, output: &PathBuf, seed: Mnemonic) {
        let wallet = self.wallet(seed);
        let input =
            std::fs::read(&input).unwrap_or_else(|_| exit_with_error("Unable to open input file."));
        let mut psbt: PartiallySignedTransaction = deserialize(&input).unwrap_or_else(|_| {
            exit_with_error("Error reading partially signed Bitcoin transction file.")
        });
        let signed = wallet
            .sign(&mut psbt, Default::default())
            .unwrap_or_else(|_| {
                exit_with_error("Error encountered while signing the transaction.")
            });

        let s = serialize(&psbt);
        std::fs::write(output, &s)
            .unwrap_or_else(|_| exit_with_error("Unable to write signed transaction."));

        if !signed {
            println!("Transaction not complete.");
        }
    }

    fn wallet(&self, seed: Mnemonic) -> Wallet<MemoryDatabase> {
        let xkey: ExtendedKey<miniscript::Segwitv0> = seed
            .into_extended_key()
            .unwrap_or_else(|_| exit_with_error("Error converting mnemonic into extended key."));
        let xprv = xkey.into_xprv(Network::Testnet).unwrap();
        Wallet::new(
            Bip84(xprv.clone(), KeychainKind::External),
            Some(Bip84(xprv.clone(), KeychainKind::External)),
            Network::Testnet,
            MemoryDatabase::default(),
        )
        .unwrap_or_else(|_| exit_with_error("Error while creating wallet."))
    }

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

    fn show_descriptor(&self, seed: Mnemonic) {
        let wallet = self.wallet(seed);
        let descriptor = wallet
            .public_descriptor(KeychainKind::External)
            .unwrap_or_else(|_| exit_with_error("Unable to obtain descriptor for wallet."))
            .unwrap_or_else(|| exit_with_error("Missing descriptor for wallet."));
        print!("{descriptor}");
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Action {
    /// Generate a mnemonic seed phrase.
    Seed {
        #[clap(short, long, help = "Output to file")]
        output: Option<PathBuf>,
    },

    /// Sign a bitcoin transaction file.
    Sign { input: PathBuf, output: PathBuf },

    /// Show wallet descriptor.
    Descriptor,
}
