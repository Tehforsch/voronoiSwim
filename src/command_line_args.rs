use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0")]
pub struct CommandLineArgs {
    pub grid_file: PathBuf,
}
