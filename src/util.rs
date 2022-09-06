pub fn exit_with_error(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}
