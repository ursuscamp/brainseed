use crate::{cli::Cli, generator::Generator};

pub fn exit_with_error(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

pub fn show_only_normalize(cli: &Cli, gen: &Generator) {
    if cli.normalized_only {
        let s = std::str::from_utf8(gen.data());

        if let Ok(s) = s {
            let s = gen.normalize(s);
            println!("{s}");
            std::process::exit(0);
        } else {
            exit_with_error("Binary data cannot be normalized.");
        }
    }
}
