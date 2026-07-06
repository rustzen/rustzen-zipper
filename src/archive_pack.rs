use crate::error::ZipperResult;
use crate::{CompressionMode, OverwriteMode};
use chrono::Local;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

#[derive(Clone, Debug, Default)]
pub struct PackRequest {
    pub source: Option<String>,
    pub output: Option<String>,
    pub format: Option<String>,
    pub compression: Option<CompressionMode>,
    pub level: Option<u8>,
    pub output_dir: Option<String>,
    pub excludes: Vec<String>,
    pub includes: Vec<String>,
    pub prefix: bool,
    pub no_prefix: bool,
    pub overwrite: Option<OverwriteMode>,
    pub sha256: bool,
    pub no_sha256: bool,
    pub base_dir: Option<String>,
    pub strip_prefix: Option<String>,
    pub dry_run: bool,
    pub no_dry_run: bool,
    pub quiet: bool,
    pub no_quiet: bool,
    pub verbose: bool,
    pub no_verbose: bool,
    pub config_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PackRuntimeOptions {
    pub source: String,
    pub output: Option<String>,
    pub format: String,
    pub compression: CompressionMode,
    pub level: u8,
    pub output_dir: String,
    pub excludes: Vec<String>,
    pub includes: Vec<String>,
    pub no_prefix: bool,
    pub sha256: bool,
    pub base_dir: Option<String>,
    pub strip_prefix: Option<String>,
    pub overwrite: OverwriteMode,
    pub dry_run: bool,
    pub quiet: bool,
    pub verbose: bool,
}

impl Default for PackRuntimeOptions {
    fn default() -> Self {
        Self {
            source: String::new(),
            output: None,
            format: "%Y%m%d-%H%M".to_string(),
            compression: CompressionMode::Deflated,
            level: 6,
            output_dir: ".".to_string(),
            excludes: Vec::new(),
            includes: Vec::new(),
            no_prefix: false,
            sha256: false,
            base_dir: None,
            strip_prefix: None,
            overwrite: OverwriteMode::Overwrite,
            dry_run: false,
            quiet: false,
            verbose: false,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PackSummary {
    pub output_path: String,
    pub total_candidates: usize,
    pub included_entries: usize,
    pub included_files: usize,
    pub included_dirs: usize,
    pub skipped_entries: usize,
    pub source_bytes: u64,
    pub included_bytes: u64,
    pub output_bytes: u64,
    pub dry_run: bool,
}

#[derive(Clone, Default)]
struct ZipStats {
    total_candidates: usize,
    included_entries: usize,
    included_files: usize,
    included_dirs: usize,
    skipped_entries: usize,
    source_bytes: u64,
    included_bytes: u64,
    output_bytes: u64,
}

pub fn pack_archive(args: PackRuntimeOptions) -> ZipperResult<PackSummary> {
    let start_time = std::time::Instant::now();
    let source = Path::new(&args.source);
    if !source.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source directory does not exist: {}", args.source),
        )
        .into());
    }
    if !source.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Source is not a directory: {}", args.source),
        )
        .into());
    }

    let include_patterns = parse_patterns(&args.includes, "include")?;
    let exclude_patterns = parse_patterns(&args.excludes, "exclude")?;
    fs::create_dir_all(&args.output_dir)?;

    let timestamp = Local::now().format(&args.format).to_string();
    let output_name = args.output.clone().unwrap_or_else(|| {
        source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let zip_path = Path::new(&args.output_dir).join(format!("{}-{}.zip", output_name, timestamp));

    if zip_path.exists() {
        match args.overwrite {
            OverwriteMode::Overwrite => {}
            OverwriteMode::Skip => {
                if !args.quiet {
                    println!("Skipped: output already exists ({})", zip_path.display());
                }

                return Ok(PackSummary {
                    output_path: zip_path.display().to_string(),
                    dry_run: args.dry_run,
                    ..PackSummary::default()
                });
            }
            OverwriteMode::Error => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("Output already exists: {}", zip_path.display()),
                )
                .into());
            }
        }
    }

    let compression_method = match args.compression {
        CompressionMode::Stored => zip::CompressionMethod::Stored,
        CompressionMode::Deflated => zip::CompressionMethod::Deflated,
    };
    let file_opts = base_file_options(&compression_method, args.level, args.compression);
    let dir_opts = SimpleFileOptions::default()
        .compression_method(compression_method)
        .unix_permissions(0o755);
    let zip_prefix = args
        .base_dir
        .as_deref()
        .map(|base| base.replace('\\', "/"))
        .filter(|name| !name.trim().is_empty())
        .or_else(|| {
            if args.no_prefix {
                None
            } else {
                Some(
                    source
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .replace('\\', "/"),
                )
            }
        });

    let mut entries = Vec::new();
    let mut stats = ZipStats::default();
    for entry in WalkDir::new(&args.source) {
        let entry = entry?;
        stats.total_candidates += 1;
        let path = entry.path();
        let rel = match path.strip_prefix(source) {
            Ok(path) => path,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let raw_rel_str = rel.to_string_lossy().replace('\\', "/");
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        if !is_included(
            &include_patterns,
            &exclude_patterns,
            &raw_rel_str,
            &file_name,
        ) {
            stats.skipped_entries += 1;
            continue;
        }
        let stripped_rel_str = strip_prefix(&raw_rel_str, args.strip_prefix.as_deref());
        let final_name = if stripped_rel_str.is_empty() {
            file_name
        } else {
            stripped_rel_str
        };
        let archive_name = match zip_prefix.as_deref() {
            None => final_name,
            Some(prefix) => format!("{prefix}/{final_name}"),
        };
        entries.push((path.to_path_buf(), archive_name, path.is_file()));
        stats.included_entries += 1;
        if path.is_dir() {
            stats.included_dirs += 1;
        } else if path.is_file() {
            stats.included_files += 1;
            stats.source_bytes += path.metadata().map(|meta| meta.len()).unwrap_or(0);
        }
    }

    if args.dry_run {
        print_summary(
            &zip_path,
            &stats,
            start_time.elapsed(),
            false,
            args.quiet,
            "preview-only",
        );
        return Ok(to_summary(&zip_path, &stats, true));
    }

    let file = File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    for (path, archive_name, is_file) in entries {
        if is_file {
            zip.start_file(&archive_name, file_opts)?;
            let mut input = File::open(path)?;
            stats.included_bytes += io::copy(&mut input, &mut zip)?;
        } else {
            zip.add_directory(format!("{}/", archive_name.trim_end_matches('/')), dir_opts)?;
        }
    }
    zip.finish()?;
    stats.output_bytes = fs::metadata(&zip_path)?.len();
    if args.sha256 {
        write_zip_sha256(&zip_path)?;
    }
    print_summary(
        &zip_path,
        &stats,
        start_time.elapsed(),
        true,
        args.quiet,
        "",
    );
    Ok(to_summary(&zip_path, &stats, false))
}

fn to_summary(zip_path: &Path, stats: &ZipStats, dry_run: bool) -> PackSummary {
    PackSummary {
        output_path: zip_path.display().to_string(),
        total_candidates: stats.total_candidates,
        included_entries: stats.included_entries,
        included_files: stats.included_files,
        included_dirs: stats.included_dirs,
        skipped_entries: stats.skipped_entries,
        source_bytes: stats.source_bytes,
        included_bytes: stats.included_bytes,
        output_bytes: stats.output_bytes,
        dry_run,
    }
}

fn base_file_options(
    method: &zip::CompressionMethod,
    level: u8,
    mode: CompressionMode,
) -> SimpleFileOptions {
    let options = SimpleFileOptions::default()
        .compression_method(*method)
        .unix_permissions(0o644);
    if let CompressionMode::Deflated = mode {
        options.compression_level(Some(i64::from(level)))
    } else {
        options
    }
}

fn parse_patterns(patterns: &[String], kind: &str) -> ZipperResult<Vec<Pattern>> {
    let mut parsed = Vec::with_capacity(patterns.len());
    for pattern in patterns {
        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            continue;
        }
        let pattern_text = if has_wildcard(trimmed) {
            trimmed.to_string()
        } else {
            format!("*{trimmed}*")
        };
        parsed.push(Pattern::new(&pattern_text).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid {kind} pattern '{trimmed}': {error}"),
            )
        })?);
    }
    Ok(parsed)
}

fn strip_prefix(path: &str, prefix: Option<&str>) -> String {
    let Some(prefix) = prefix else {
        return path.to_string();
    };
    let normalized_prefix = prefix
        .trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();
    if normalized_prefix.is_empty() {
        return path.to_string();
    }
    if path == normalized_prefix {
        return String::new();
    }
    let expected = format!("{normalized_prefix}/");
    path.strip_prefix(&expected).unwrap_or(path).to_string()
}

fn write_zip_sha256(zip_path: &Path) -> ZipperResult<()> {
    let mut hasher = Sha256::new();
    let mut input = File::open(zip_path)?;
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let size = input.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hasher.update(&buffer[..size]);
    }
    let encoded = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut output = File::create(zip_path.with_extension("zip.sha256"))?;
    let file_name = zip_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("archive.zip");
    output.write_all(format!("{encoded}  {file_name}\n").as_bytes())?;
    Ok(())
}

fn has_wildcard(pattern: &str) -> bool {
    pattern.chars().any(|c| matches!(c, '*' | '?' | '[' | ']'))
}

fn is_included(
    includes: &[Pattern],
    excludes: &[Pattern],
    rel_path: &str,
    file_name: &str,
) -> bool {
    if !includes.is_empty()
        && !includes
            .iter()
            .any(|pattern| matches_pattern(pattern, rel_path, file_name))
    {
        return false;
    }
    !excludes
        .iter()
        .any(|pattern| matches_pattern(pattern, rel_path, file_name))
}

fn matches_pattern(pattern: &Pattern, rel_path: &str, file_name: &str) -> bool {
    pattern.matches(rel_path) || pattern.matches(file_name)
}

fn print_summary(
    zip_path: &Path,
    stats: &ZipStats,
    elapsed: Duration,
    with_output: bool,
    quiet: bool,
    mode: &str,
) {
    if quiet {
        return;
    }
    if !mode.is_empty() {
        println!("Mode: {mode}");
    }
    if with_output {
        println!("Created zip: {}", zip_path.display());
    }
    let ratio = if stats.source_bytes > 0 && stats.output_bytes > 0 {
        (100.0 - (stats.output_bytes as f64 / stats.source_bytes as f64 * 100.0)).max(0.0)
    } else {
        0.0
    };
    println!("Files: {}", stats.included_files);
    println!("Source bytes: {} B", stats.source_bytes);
    println!("Archive bytes: {} B", stats.output_bytes);
    println!("Elapsed: {:?}", elapsed);
    println!(
        "Entries: {} (dirs: {}, files: {})",
        stats.included_entries, stats.included_dirs, stats.included_files
    );
    println!("Skipped: {}", stats.skipped_entries);
    println!("Approx compression ratio: {:.2}%", ratio);
}
