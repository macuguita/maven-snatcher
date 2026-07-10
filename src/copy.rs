use crate::args::CopyArgs;
use crate::common::{DestContext, make_progress_bar, run_parallel};
use anyhow::{Result, bail};
use reqwest::StatusCode;
use reqwest::blocking::Client;

struct Coordinate {
    group_id: String,
    artifact_id: String,
    version: String,
}

impl Coordinate {
    fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 3 {
            bail!(
                "invalid coordinate '{}', expected group:artifact:version",
                s
            );
        }
        Ok(Self {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: parts[2].to_string(),
        })
    }

    fn dir(&self) -> String {
        format!(
            "{}/{}/{}/",
            self.group_id.replace('.', "/"),
            self.artifact_id,
            self.version
        )
    }

    fn candidate_files(&self, include_hashes: bool) -> Vec<String> {
        let base = format!("{}-{}", self.artifact_id, self.version);
        let mut files = vec![
            format!("{base}.pom"),
            format!("{base}.jar"),
            format!("{base}-sources.jar"),
            format!("{base}-javadoc.jar"),
        ];

        if include_hashes {
            let mut with_hashes = Vec::new();
            for f in &files {
                with_hashes.push(format!("{f}.md5"));
                with_hashes.push(format!("{f}.sha1"));
            }
            files.extend(with_hashes);
        }
        files
    }
}

struct SourceContext {
    client: Client,
    base_url: String,
}

pub fn run(args: CopyArgs) -> Result<()> {
    let coords: Vec<Coordinate> = args
        .coordinates
        .iter()
        .map(|s| Coordinate::parse(s))
        .collect::<Result<_>>()?;

    let src = SourceContext {
        client: Client::builder()
            .user_agent("maven-worker-migrate")
            .build()?,
        base_url: args.source_url.trim_end_matches('/').to_owned(),
    };
    let dst = DestContext::new(
        args.dest.url,
        args.dest.username,
        args.dest.password,
        args.dest.dry_run,
    )?;

    println!("Copying {} coordinate(s)...", coords.len());
    let progress = make_progress_bar(coords.len() as u64);

    let result = run_parallel(
        &coords,
        args.dest.threads,
        args.dest.continue_on_error,
        &progress,
        |coord| copy_coordinate(&src, &dst, coord, args.include_hashes, &progress),
    );

    progress.finish();
    result?;
    println!("Copy complete!");
    Ok(())
}

fn copy_coordinate(
    src: &SourceContext,
    dst: &DestContext,
    coord: &Coordinate,
    include_hashes: bool,
    progress: &indicatif::ProgressBar,
) -> Result<()> {
    let dir = coord.dir();

    for filename in coord.candidate_files(include_hashes) {
        let rel_path = format!("{dir}{filename}");
        let src_url = format!("{}/{}", src.base_url, rel_path);

        let resp = src.client.get(&src_url).send()?;
        if resp.status() == StatusCode::NOT_FOUND {
            continue; // optional artifact (e.g. sources/javadoc), fine to skip
        }
        if !resp.status().is_success() {
            bail!("GET {} returned {}", src_url, resp.status());
        }
        let bytes = resp.bytes()?.to_vec();

        progress.println(format!("Copying {rel_path}"));
        dst.put(&rel_path, bytes)?;
    }

    Ok(())
}
