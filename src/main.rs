mod cli;
mod generator;
mod util;

use clap::Parser;
use cli::Cli;
use generator::Generator;

fn main() {
    let cli = Cli::parse();

    let mut gen = Generator::from(cli.clone());
    let seed = gen.seed();
    cli.write_output(seed.to_string().as_bytes());
}
