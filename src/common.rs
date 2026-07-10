use anyhow::{Result, bail};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::sync::Mutex;

pub struct DestContext {
    pub client: Client,
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub dry_run: bool,
}

impl DestContext {
    pub fn new(url: String, username: String, password: String, dry_run: bool) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .user_agent("maven-worker-migrate")
                .build()?,
            base_url: url.trim_end_matches('/').to_owned(),
            username,
            password,
            dry_run,
        })
    }

    pub fn put(&self, relative_path: &str, bytes: Vec<u8>) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        let url = format!("{}/{}", self.base_url, relative_path);
        let response = self
            .client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(bytes)
            .send()?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            bail!("{} returned {}\n{}", relative_path, status, body);
        }
        Ok(())
    }
}

pub fn make_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    if let Ok(style) = ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} ({eta}) {msg}")
    {
        pb.set_style(style.progress_chars("=>-"));
    }
    pb
}

/// Runs `f` over `items` on a rayon pool sized to `threads`.
/// If `continue_on_error` is false, stops at the first error (fail-fast).
/// If true, runs everything, collects all errors, and reports them together.
pub fn run_parallel<T, F>(
    items: &[T],
    threads: usize,
    continue_on_error: bool,
    progress: &ProgressBar,
    f: F,
) -> Result<()>
where
    T: Sync,
    F: Fn(&T) -> Result<()> + Sync,
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()?;

    if continue_on_error {
        let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
        pool.install(|| {
            items.par_iter().for_each(|item| {
                if let Err(e) = f(item) {
                    errors.lock().unwrap().push(e.to_string());
                }
                progress.inc(1);
            });
        });
        let errors = errors.into_inner().unwrap();
        if !errors.is_empty() {
            for e in &errors {
                eprintln!("error: {e}");
            }
            bail!("{} item(s) failed", errors.len());
        }
        Ok(())
    } else {
        pool.install(|| {
            items.par_iter().try_for_each(|item| {
                let res = f(item);
                progress.inc(1);
                res
            })
        })
    }
}
