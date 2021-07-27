use log::*;

mod mca_file;
use mca_file::McaFile;

fn main() {

    simple_logging::log_to_file("cbgen.log", LevelFilter::max()).unwrap_or_else(|_| {
        simple_logging::log_to(std::io::stdout(), LevelFilter::max());
    });
    log_panics::init();

    // let f = File::open("test.0.0.mca").unwrap_or_else(|_| {
    //     println!("Got the wrong path");
    //     std::process::exit(1);
    // });

    let mca = McaFile::open("test.0.0.mca").unwrap_or_else(|_| {
        println!("No test mca file at this path");
        std::process::exit(1);
    });

}
