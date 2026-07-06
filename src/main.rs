use clap::{ArgAction, Parser, Subcommand};
use rustzen_zipper::config::resolve_pack_options;
use rustzen_zipper::inspect::{InspectOptions, inspect_archive, print_inspect_summary};
use rustzen_zipper::pack::{PackRequest, pack_archive};
use rustzen_zipper::unpack::{UnpackOptions, unpack_archive};
use rustzen_zipper::{CompressionMode, OverwriteMode};

#[derive(Parser, Debug)]
#[command(name = "rz-zip", bin_name = "rz-zip")]
#[command(about = "A Rust-based CLI and macOS app core for zipping and unzipping archives")]
#[command(version)]
struct Cli {
    /// Default behavior without a subcommand: archive files into a zip.
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    pack: PackCliOptions,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Archive files into a zip.
    Pack(PackCliOptions),
    /// Unpack files from a zip archive.
    Unpack(UnpackCliOptions),
    /// Inspect zip archive entries without extracting them.
    Inspect(InspectCliOptions),
}

#[derive(clap::Args, Clone, Debug, Default)]
struct PackCliOptions {
    /// Source directory to zip.
    #[arg(short, long)]
    source: Option<String>,

    /// Output zip file name without extension.
    #[arg(short, long)]
    output: Option<String>,

    /// Time format for timestamp.
    #[arg(short, long)]
    format: Option<String>,

    /// Compression method.
    #[arg(short, long, value_enum)]
    compression: Option<CompressionMode>,

    /// Compression level. Use 0..=9, only effective for deflated.
    #[arg(short = 'l', long, value_parser = clap::value_parser!(u8).range(0..=9))]
    level: Option<u8>,

    /// Output directory for the zip file.
    #[arg(short = 'd', long)]
    output_dir: Option<String>,

    /// Paths to exclude. Glob patterns supported. Can be passed multiple times.
    #[arg(short = 'x', long = "exclude", short_alias = 'e', alias = "excludes")]
    excludes: Vec<String>,

    /// Paths to include. Glob patterns supported. Can be passed multiple times.
    #[arg(short = 'i', long = "include", alias = "includes")]
    includes: Vec<String>,

    /// Keep source root directory name in zip path.
    #[arg(long = "prefix", action = ArgAction::SetTrue)]
    prefix: bool,

    /// Do not keep source root directory name in zip path.
    #[arg(long = "no-prefix", action = ArgAction::SetTrue)]
    no_prefix: bool,

    /// Output file exists, handle as overwrite/skip/error.
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
struct UnpackCliOptions {
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

#[derive(clap::Args, Clone, Debug)]
struct InspectCliOptions {
    /// Zip archive to inspect.
    #[arg(short, long)]
    source: String,

    /// Output inspect result as JSON.
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,

    /// Mute normal logs, keep only errors.
    #[arg(short = 'q', long, action = ArgAction::SetTrue)]
    quiet: bool,
}

impl From<PackCliOptions> for PackRequest {
    fn from(value: PackCliOptions) -> Self {
        Self {
            source: value.source,
            output: value.output,
            format: value.format,
            compression: value.compression,
            level: value.level,
            output_dir: value.output_dir,
            excludes: value.excludes,
            includes: value.includes,
            prefix: value.prefix,
            no_prefix: value.no_prefix,
            overwrite: value.overwrite,
            sha256: value.sha256,
            no_sha256: value.no_sha256,
            base_dir: value.base_dir,
            strip_prefix: value.strip_prefix,
            dry_run: value.dry_run,
            no_dry_run: value.no_dry_run,
            quiet: value.quiet,
            no_quiet: value.no_quiet,
            verbose: value.verbose,
            no_verbose: value.no_verbose,
            config_path: value.config_path,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => run_pack(cli.pack),
        Some(Commands::Pack(pack_args)) => run_pack(pack_args),
        Some(Commands::Unpack(unpack_args)) => run_unpack(unpack_args),
        Some(Commands::Inspect(inspect_args)) => run_inspect(inspect_args),
    };

    if let Err(error) = result {
        eprintln!("Failed to execute command: {error}");
        std::process::exit(1);
    }
}

fn run_pack(args: PackCliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let options = resolve_pack_options(args.into())?;
    pack_archive(options)?;
    Ok(())
}

fn run_unpack(args: UnpackCliOptions) -> Result<(), Box<dyn std::error::Error>> {
    unpack_archive(UnpackOptions {
        source: args.source,
        output_dir: args.output_dir,
        quiet: args.quiet,
    })?;
    Ok(())
}

fn run_inspect(args: InspectCliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let summary = inspect_archive(InspectOptions {
        source: args.source,
        quiet: args.quiet,
        json: args.json,
    })?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_inspect_summary(&summary, args.quiet);
    }

    Ok(())
}
