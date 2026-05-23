fn main() {
    if let Err(error) = harness_lint::cli::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
