mod cli;
mod generator;
mod util;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli.exec();
}
