use chrono::Local;
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use glob::Pattern;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;

#[derive(Parser, Debug)]
#[command(name = "rz-zip", bin_name = "rz-zip")]
#[command(about = "A Rust-based CLI tool for zipping dist folders")]
#[command(version)]
struct Cli {
    /// Default behavior: archive files into a zip.
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    pack: PackOptions,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "unpack", hide = true)]
    /// Unpack files from a zip archive.
    Unpack(UnpackOptions),
}

#[derive(clap::Args, Clone, Debug)]
struct PackOptions {
    /// Source directory to zip.
    #[arg(short, long)]
    source: Option<String>,

    /// Output zip file name (without extension).
    /// If not specified, uses the source directory name.
    #[arg(short, long)]
    output: Option<String>,

    /// Time format for timestamp.
    #[arg(short, long)]
    format: Option<String>,

    /// Compression method.
    #[arg(short, long, value_enum)]
    compression: Option<CompressionMode>,

    /// Compression level. Use `0..=9`, only effective for `deflated`.
    #[arg(short = 'l', long, value_parser = clap::value_parser!(u8).range(0..=9))]
    level: Option<u8>,

    /// Output directory for the zip file.
    #[arg(short = 'd', long)]
    output_dir: Option<String>,

    /// Paths to exclude (glob patterns supported). Can be passed multiple times.
    #[arg(short = 'x', long = "exclude", short_alias = 'e', alias = "excludes")]
    excludes: Vec<String>,

    /// Paths to include (glob patterns supported). Can be passed multiple times.
    #[arg(short = 'i', long = "include", alias = "includes")]
    includes: Vec<String>,

    /// Keep source root directory name in zip path.
    #[arg(long = "prefix", action = ArgAction::SetTrue)]
    prefix: bool,

    /// Do not keep source root directory name in zip path.
    #[arg(long = "no-prefix", action = ArgAction::SetTrue)]
    no_prefix: bool,

    /// Output file exists, handle as overwrite.
    #[arg(long, value_enum)]
    overwrite: Option<OverwriteMode>,

    /// Add the archive file checksum `<name>.sha256` after packaging.
    #[arg(long, action = ArgAction::SetTrue)]
    sha256: bool,

    /// Skip generating `<name>.sha256`.
    #[arg(long = "no-sha256", action = ArgAction::SetTrue)]
    no_sha256: bool,

    /// Replace the archive root folder name.
    #[arg(long)]
    base_dir: Option<String>,

    /// Remove the matching leading archive path prefix before adding.
    #[arg(long)]
    strip_prefix: Option<String>,

    /// Only show planned operations and skip zip creation.
    #[arg(long, action = ArgAction::SetTrue)]
    dry_run: bool,

    /// Create archive instead of preview mode.
    #[arg(long = "no-dry-run", action = ArgAction::SetTrue)]
    no_dry_run: bool,

    /// Mute normal logs, keep only errors.
    #[arg(short = 'q', long, action = ArgAction::SetTrue)]
    quiet: bool,

    /// Keep normal output.
    #[arg(long = "no-quiet", action = ArgAction::SetTrue)]
    no_quiet: bool,

    /// Show detailed logs.
    #[arg(short = 'v', long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Keep verbose logs off.
    #[arg(long = "no-verbose", action = ArgAction::SetTrue)]
    no_verbose: bool,

    /// Load options from config file (`.rzrc`, `.rzrc.json`, or `package.json` `rz-zip` field).
    #[arg(long = "config")]
    config_path: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
struct UnpackOptions {
    /// Zip archive to unpack.
    #[arg(short, long)]
    source: String,

    /// Directory to extract files into.
    #[arg(short = 'o', long, default_value = ".")]
    output_dir: String,

    /// Mute normal logs, keep only errors.
    #[arg(short = 'q', long, action = ArgAction::SetTrue)]
    quiet: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CompressionMode {
    /// Store files without compression.
    Stored,
    /// Use DEFLATE compression.
    Deflated,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OverwriteMode {
    /// Overwrite output file when it already exists.
    Overwrite,
    /// Skip generating output when target file exists.
    Skip,
    /// Fail when target file exists.
    Error,
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

#[derive(Clone)]
struct PackRuntimeOptions {
    source: String,
    output: Option<String>,
    format: String,
    compression: CompressionMode,
    level: u8,
    output_dir: String,
    excludes: Vec<String>,
    includes: Vec<String>,
    no_prefix: bool,
    sha256: bool,
    base_dir: Option<String>,
    strip_prefix: Option<String>,
    overwrite: OverwriteMode,
    dry_run: bool,
    quiet: bool,
    verbose: bool,
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

#[derive(Default)]
struct PackConfig {
    source: Option<String>,
    output: Option<String>,
    format: Option<String>,
    compression: Option<CompressionMode>,
    level: Option<u8>,
    output_dir: Option<String>,
    excludes: Vec<String>,
    includes: Vec<String>,
    sha256: Option<bool>,
    base_dir: Option<String>,
    strip_prefix: Option<String>,
    no_prefix: Option<bool>,
    overwrite: Option<OverwriteMode>,
    dry_run: Option<bool>,
    quiet: Option<bool>,
    verbose: Option<bool>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => run_pack(cli.pack),
        Some(Commands::Unpack(unpack_args)) => run_unpack(unpack_args),
    };

    if let Err(error) = result {
        eprintln!("Failed to execute command: {error}");
        std::process::exit(1);
    }
}

fn run_unpack(args: UnpackOptions) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = Path::new(&args.source);
    if !source_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source zip does not exist: {source}", source = args.source),
        )
        .into());
    }

    fs::create_dir_all(&args.output_dir).map_err(|error| {
        io::Error::other(format!(
            "Failed to prepare output directory '{}': {error}",
            args.output_dir
        ))
    })?;

    let source = File::open(source_path)?;
    let mut archive = ZipArchive::new(source)?;

    for idx in 0..archive.len() {
        let mut file = archive.by_index(idx)?;
        let out_name = file
            .enclosed_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid zip entry name"))?;

        let out_path = Path::new(&args.output_dir).join(out_name);

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output = File::create(&out_path)?;
        io::copy(&mut file, &mut output)?;

        if !args.quiet {
            println!("  extracted: {}", out_path.display());
        }
    }

    if !args.quiet {
        println!("Unpacked: {} file(s)", archive.len());
    }
    Ok(())
}

fn run_pack(raw: PackOptions) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let args = resolve_pack_options(raw)?;

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

    fs::create_dir_all(&args.output_dir).map_err(|error| {
        io::Error::other(format!(
            "Failed to prepare output directory '{}': {error}",
            args.output_dir
        ))
    })?;

    let timestamp = Local::now().format(&args.format).to_string();
    let output_name = args.output.unwrap_or_else(|| {
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
                    println!("Skipped: output already exists: {}", zip_path.display());
                }
                return Ok(());
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

        let included = is_included(
            &include_patterns,
            &exclude_patterns,
            &raw_rel_str,
            &file_name,
        );
        if !args.quiet && (args.verbose || args.dry_run) {
            let status = if included { "add" } else { "skip" };
            println!("  {status}: {raw_rel_str}");
        }

        if !included {
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
        return Ok(());
    }

    let file = File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let total_steps = entries.len();

    for (idx, (path, archive_name, is_file)) in entries.into_iter().enumerate() {
        let progress = format!("[{}/{}]", idx + 1, total_steps);
        if is_file {
            if args.verbose && !args.quiet {
                println!("  {progress} file: {archive_name}");
            }
            zip.start_file(&archive_name, file_opts)?;
            let mut input = File::open(path)?;
            let copied = io::copy(&mut input, &mut zip)?;
            stats.included_bytes += copied;
        } else {
            if args.verbose && !args.quiet {
                println!("  {progress} dir: {archive_name}");
            }
            let name = archive_name.trim_end_matches('/').to_string();
            zip.add_directory(format!("{name}/"), dir_opts)?;
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

    Ok(())
}

fn resolve_pack_options(
    raw: PackOptions,
) -> Result<PackRuntimeOptions, Box<dyn std::error::Error>> {
    let config = load_pack_config(raw.config_path.as_deref())?;

    Ok(PackRuntimeOptions {
        source: raw
            .source
            .or(config.source)
            .unwrap_or_else(|| "./dist".to_string()),
        output: raw.output.or(config.output),
        format: raw
            .format
            .or(config.format)
            .unwrap_or_else(|| "%Y%m%d-%H%M".to_string()),
        compression: raw
            .compression
            .or(config.compression)
            .unwrap_or(CompressionMode::Deflated),
        level: raw.level.or(config.level).unwrap_or(6),
        output_dir: raw
            .output_dir
            .or(config.output_dir)
            .unwrap_or_else(|| ".".to_string()),
        excludes: if raw.excludes.is_empty() {
            config.excludes
        } else {
            raw.excludes
        },
        includes: if raw.includes.is_empty() {
            config.includes
        } else {
            raw.includes
        },
        sha256: if raw.no_sha256 {
            false
        } else if raw.sha256 {
            true
        } else {
            config.sha256.unwrap_or(false)
        },
        base_dir: raw.base_dir.or(config.base_dir),
        strip_prefix: raw.strip_prefix.or(config.strip_prefix),
        no_prefix: if raw.prefix {
            false
        } else if raw.no_prefix {
            true
        } else {
            config.no_prefix.unwrap_or(false)
        },
        overwrite: raw
            .overwrite
            .or(config.overwrite)
            .unwrap_or(OverwriteMode::Overwrite),
        dry_run: if raw.no_dry_run {
            false
        } else if raw.dry_run {
            true
        } else {
            config.dry_run.unwrap_or(false)
        },
        quiet: if raw.no_quiet {
            false
        } else if raw.quiet {
            true
        } else {
            config.quiet.unwrap_or(false)
        },
        verbose: if raw.no_verbose {
            false
        } else if raw.verbose {
            true
        } else {
            config.verbose.unwrap_or(false)
        },
    })
}

fn load_pack_config(config_path: Option<&str>) -> Result<PackConfig, Box<dyn std::error::Error>> {
    let candidates = if let Some(config_path) = config_path {
        let explicit = PathBuf::from(config_path);
        if !explicit.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Config file does not exist: {}", explicit.display()),
            )
            .into());
        }

        vec![explicit]
    } else {
        let cwd = std::env::current_dir()?;
        vec![
            cwd.join(".rzrc"),
            cwd.join(".rzrc.json"),
            cwd.join("package.json"),
        ]
    };

    for candidate in candidates {
        if candidate.exists() {
            let cfg = read_pack_config_file(&candidate)?;
            if let Some(cfg) = cfg {
                return Ok(cfg);
            }
        }
    }

    Ok(PackConfig::default())
}

fn read_pack_config_file(path: &Path) -> Result<Option<PackConfig>, Box<dyn std::error::Error>> {
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;

    if content.trim().is_empty() {
        return Ok(Some(PackConfig::default()));
    }

    let value: Value = serde_json::from_str(&content)?;

    let config_obj = if path.file_name().and_then(|name| name.to_str()) == Some("package.json") {
        extract_packagejson_pack_config(&value)
    } else {
        Some(value)
    };

    let Some(config_obj) = config_obj else {
        return Ok(None);
    };

    Ok(Some(parse_pack_config(&config_obj)?))
}

fn extract_packagejson_pack_config(value: &Value) -> Option<Value> {
    value.get("rz-zip").cloned()
}

fn parse_pack_config(value: &Value) -> Result<PackConfig, Box<dyn std::error::Error>> {
    let as_obj = value.as_object();
    if as_obj.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Config root must be a JSON object",
        )
        .into());
    }
    let value = as_obj.unwrap();

    Ok(PackConfig {
        source: value
            .get("source")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        output: value
            .get("output")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        format: value
            .get("format")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        compression: value
            .get("compression")
            .and_then(|v| v.as_str())
            .map(|v| parse_compression_mode("compression", v))
            .transpose()?,
        level: value.get("level").map(parse_level).transpose()?,
        output_dir: value
            .get("output_dir")
            .or_else(|| value.get("output-dir"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        excludes: parse_string_array(value.get("excludes").or_else(|| value.get("exclude"))),
        includes: parse_string_array(value.get("includes").or_else(|| value.get("include"))),
        sha256: value
            .get("sha256")
            .map(|v| parse_bool("sha256", v))
            .transpose()?,
        base_dir: value
            .get("base_dir")
            .or_else(|| value.get("base-dir"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        strip_prefix: value
            .get("strip_prefix")
            .or_else(|| value.get("strip-prefix"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        no_prefix: value
            .get("no_prefix")
            .map(|v| parse_bool("no_prefix", v))
            .transpose()?,
        overwrite: value
            .get("overwrite")
            .and_then(|v| v.as_str())
            .map(|v| parse_overwrite_mode("overwrite", v))
            .transpose()?,
        dry_run: value
            .get("dry_run")
            .map(|v| parse_bool("dry_run", v))
            .transpose()?,
        quiet: value
            .get("quiet")
            .map(|v| parse_bool("quiet", v))
            .transpose()?,
        verbose: value
            .get("verbose")
            .map(|v| parse_bool("verbose", v))
            .transpose()?,
    })
}

fn parse_string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_compression_mode(
    key: &str,
    value: &str,
) -> Result<CompressionMode, Box<dyn std::error::Error>> {
    match value.to_ascii_lowercase().as_str() {
        "stored" => Ok(CompressionMode::Stored),
        "deflated" => Ok(CompressionMode::Deflated),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid {key} value '{value}'"),
        )
        .into()),
    }
}

fn parse_level(value: &Value) -> Result<u8, Box<dyn std::error::Error>> {
    let level = value.as_u64().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "`level` must be an integer in range 0..=9",
        )
    })?;

    let level = u8::try_from(level).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "`level` must be an integer in range 0..=9",
        )
    })?;

    if (0..=9).contains(&level) {
        Ok(level)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "`level` must be in range 0..=9",
        )
        .into())
    }
}

fn parse_overwrite_mode(
    key: &str,
    value: &str,
) -> Result<OverwriteMode, Box<dyn std::error::Error>> {
    match value.to_ascii_lowercase().as_str() {
        "overwrite" => Ok(OverwriteMode::Overwrite),
        "skip" => Ok(OverwriteMode::Skip),
        "error" => Ok(OverwriteMode::Error),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid {key} value '{value}'"),
        )
        .into()),
    }
}

fn parse_bool(key: &str, value: &Value) -> Result<bool, Box<dyn std::error::Error>> {
    value.as_bool().ok_or_else(|| {
        {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("`{key}` must be a boolean value"),
            )
        }
        .into()
    })
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

fn parse_patterns(
    patterns: &[String],
    kind: &str,
) -> Result<Vec<Pattern>, Box<dyn std::error::Error>> {
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

        let compiled = Pattern::new(&pattern_text).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid {kind} pattern '{trimmed}': {error}"),
            )
        })?;
        parsed.push(compiled);
    }

    Ok(parsed)
}

fn strip_prefix(path: &str, prefix: Option<&str>) -> String {
    let normalized_prefix = match prefix {
        Some(prefix) => prefix.trim(),
        None => return path.to_string(),
    };

    if normalized_prefix.is_empty() {
        return path.to_string();
    }

    let normalized_prefix = normalized_prefix
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
    if let Some(rest) = path.strip_prefix(&expected) {
        return rest.to_string();
    }

    path.to_string()
}

fn write_zip_sha256(zip_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
    let hash = hasher.finalize();

    let encoded = hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let sha_path = zip_path.with_extension("zip.sha256");
    let mut output = File::create(&sha_path)?;
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
    if !includes.is_empty() {
        let matched_include = includes
            .iter()
            .any(|pattern| matches_pattern(pattern, rel_path, file_name));
        if !matched_include {
            return false;
        }
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
        let ratio = 100.0 - (stats.output_bytes as f64 / stats.source_bytes as f64 * 100.0);
        if ratio < 0.0 { 0.0 } else { ratio }
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
