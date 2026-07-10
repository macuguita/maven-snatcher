use crate::args::MigrateArgs;
use crate::common::{DestContext, make_progress_bar, run_parallel};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn run(args: &MigrateArgs) -> Result<()> {
    let ctx = DestContext::new(
        &args.dest.url,
        &args.dest.username,
        &args.dest.password,
        args.dest.dry_run,
    )?;

    let files = collect_files(&args.directory);
    if files.is_empty() {
        println!("No files found to upload.");
        return Ok(());
    }
    println!("Found {} files to upload.", files.len());

    let progress = make_progress_bar(files.len() as u64);

    let result = run_parallel(
        &files,
        args.dest.threads,
        args.dest.continue_on_error,
        &progress,
        |file| upload(&ctx, &args.directory, file, &progress),
    );

    progress.finish();
    result?;
    println!("Migration complete!");
    Ok(())
}

fn collect_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| should_upload(path))
        .collect()
}

fn should_upload(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    !(name.ends_with(".md5")
        || name.ends_with(".sha1")
        || name.ends_with(".sha256")
        || name.ends_with(".sha512")
        || name == "maven-metadata.xml")
}

fn upload(
    ctx: &DestContext,
    root: &Path,
    file: &Path,
    progress: &indicatif::ProgressBar,
) -> Result<()> {
    let relative = file
        .strip_prefix(root)
        .context("failed to compute relative path")?;
    let relative_str = relative.to_string_lossy().replace('\\', "/");

    progress.println(format!("Uploading {relative_str}"));

    let bytes = fs::read(file)?;
    ctx.put(&relative_str, bytes)
}
