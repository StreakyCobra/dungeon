use dungeon::app;

fn main() {
    if let Err(err) = app::run() {
        eprintln!("{}", err);
        std::process::exit(err.exit_code());
    }
}
