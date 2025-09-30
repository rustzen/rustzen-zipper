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
    #[arg(short, long, default_value = "dist")]
    output: String,

    /// Time format for timestamp
    #[arg(short, long, default_value = "%Y%m%d-%H%M")]
    format: String,

    /// Compression method
    #[arg(short, long, default_value = "stored")]
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
    let zip_name = format!("{}-{}.zip", args.output, timestamp);

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

    // 遍历 dist，并显式写入“目录项”，从而保留空目录结构
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

        // ZIP 规范更偏好正斜杠；同时对非 UTF-8 路径做安全降级
        let name = rel.to_string_lossy().replace('\\', "/");

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
