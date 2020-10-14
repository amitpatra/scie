use clap::Clap;
use crate::validate::Validate;
use scie_core::analyser::Analyser;
use std::path::{PathBuf, Path};

pub mod validate;

#[derive(Clap)]
#[clap(version = "0.1", author = "Phodal HUANG<h@phodal.com>")]
struct Opts {
    #[clap(short, long, default_value = "default.conf")]
    config: String,
    #[clap(short, long, default_value = ".")]
    path: String,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() {
    let opts: Opts = Opts::parse();

    // println!("Value for config: {}", opts.config);
    println!("Using input file: {}", opts.path);

    if !Validate::is_valid_path(opts.path.clone()) {
        println!("error");
        return;
    }

    let path = Path::new(&opts.path);
    Analyser::ident_by_dir(&path.to_path_buf());
}
