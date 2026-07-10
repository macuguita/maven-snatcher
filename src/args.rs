use clap::{Args as ClapArgs, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "maven-worker-migrate")]
#[command(version)]
#[command(about = "Migrates and copies artifacts for xander-maven-worker")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Migrate a local maven directory to a remote repo
    Migrate(MigrateArgs),
    /// Copy specific coordinates from another maven repo into yours
    Copy(CopyArgs),
}

#[derive(ClapArgs, Debug)]
pub struct MigrateArgs {
    #[arg(short, long)]
    pub directory: PathBuf,

    #[command(flatten)]
    pub dest: DestArgs,
}

#[derive(ClapArgs, Debug)]
pub struct CopyArgs {
    /// Source maven base URL, e.g. <https://maven.fabricmc.net>
    #[arg(long)]
    pub source_url: String,

    /// Coordinates in group:artifact:version form, can be passed multiple times
    #[arg(short = 'c', long = "coordinate", required = true)]
    pub coordinates: Vec<String>,

    /// Also copy .md5/.sha1 checksum files instead of relying on dest auto-generation
    #[arg(long)]
    pub include_hashes: bool,

    #[command(flatten)]
    pub dest: DestArgs,
}

#[derive(ClapArgs, Debug)]
pub struct DestArgs {
    #[arg(long)]
    pub url: String,

    #[arg(short, long)]
    pub username: String,

    #[arg(short, long)]
    pub password: String,

    #[arg(short, long, default_value_t = 8)]
    pub threads: usize,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub continue_on_error: bool,
}
