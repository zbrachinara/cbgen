use log::*;

mod mca_file;
use mca_file::McaFile;

fn main() {
    simple_logging::log_to_file("cbgen.log", LevelFilter::max()).unwrap_or_else(|_| {
        simple_logging::log_to(std::io::stdout(), LevelFilter::max());
    });
    log_panics::init();

    let filename = std::env::args().nth(1).unwrap_or("test.0.0.mca".into());

    let mca = McaFile::open(filename).unwrap_or_else(|_| {
        error!("No test mca file at this path");
        std::process::exit(1);
    });
}
