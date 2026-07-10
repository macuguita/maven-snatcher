mod args;
mod common;
mod copy;
mod migrate;

use anyhow::Result;
use args::{Args, Command};
use clap::Parser;

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Migrate(a) => migrate::run(a),
        Command::Copy(a) => copy::run(a),
    }
}
