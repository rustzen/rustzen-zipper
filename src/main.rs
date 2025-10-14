use chrono::Local;
use clap::Parser;
use std::fs::File;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

#[derive(Parser, Debug)]
#[command(name = "@rustzen/zipper")]
#[command(about = "A Rust-based CLI tool for zipping dist folders")]
#[command(version)]
struct Args {
    /// Source directory to zip
    #[arg(short, long, default_value = "./dist")]
    source: String,

    /// Output zip file name (without extension)
    /// If not specified, will use the source directory name
    #[arg(short, long)]
    output: Option<String>,

    /// Time format for timestamp
    #[arg(short, long, default_value = "%Y%m%d-%H%M")]
    format: String,

    /// Compression method
    /// Type: String (stored|deflated)
    #[arg(short, long, default_value = "deflated")]
    compression: String,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("Failed to create zip: {e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> zip::result::ZipResult<()> {
    let start_time = std::time::Instant::now();

    // 生成时间戳
    let timestamp = Local::now().format(&args.format).to_string();

    // 自动从 source 路径提取目录名作为默认输出名
    let output_name = args.output.unwrap_or_else(|| {
        std::path::Path::new(&args.source)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let zip_name = format!("{}-{}.zip", output_name, timestamp);

    // 创建 zip 写入器
    let file = File::create(&zip_name)?;
    let mut zip = zip::ZipWriter::new(file);

    // 压缩设置：
    // - 默认使用 Stored（无压缩，无需启用额外 feature）
    // - 如果要开启压缩（如 Deflated），需在 Cargo.toml 启用相应 feature
    let compression_method = match args.compression.as_str() {
        "stored" => zip::CompressionMethod::Stored,
        "deflated" => zip::CompressionMethod::Deflated,
        _ => zip::CompressionMethod::Stored,
    };
    let file_opts = SimpleFileOptions::default()
        .compression_method(compression_method)
        .unix_permissions(0o644);
    let dir_opts = SimpleFileOptions::default()
        .compression_method(compression_method)
        .unix_permissions(0o755);

    // 遍历 dist，并显式写入“目录项”
    for entry in WalkDir::new(&args.source)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();

        // 计算 zip 内部的相对路径；根目录条目跳过
        let rel = match path.strip_prefix(&args.source) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }

        // 构建包含 dist 目录的完整路径
        let name = if rel.as_os_str().is_empty() {
            // 如果是 dist 目录本身，直接使用 "dist"
            args.source.to_string()
        } else {
            // 其他文件和目录，添加 "dist/" 前缀
            format!(
                "{}/{}",
                args.source,
                rel.to_string_lossy().replace('\\', "/")
            )
        };

        if path.is_dir() {
            // 显式添加目录项，保证某些解压工具能恢复空目录
            zip.add_directory(&name, dir_opts)?;
        } else if path.is_file() {
            // 添加文件内容
            zip.start_file(&name, file_opts)?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        }
    }

    zip.finish()?;
    println!("Created zip: {}", zip_name);
    println!("Time taken: {:?}", start_time.elapsed());
    Ok(())
}
