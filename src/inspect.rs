use crate::error::ZipperResult;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::Path;
use zip::read::ZipArchive;

#[derive(Clone, Debug)]
pub struct InspectOptions {
    pub source: String,
    pub quiet: bool,
    pub json: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub is_dir: bool,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct InspectSummary {
    pub source: String,
    pub entries: Vec<ArchiveEntry>,
    pub file_count: usize,
    pub dir_count: usize,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

pub fn inspect_archive(args: InspectOptions) -> ZipperResult<InspectSummary> {
    let source_path = Path::new(&args.source);
    if !source_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source zip does not exist: {}", args.source),
        )
        .into());
    }

    let source = File::open(source_path)?;
    let mut archive = ZipArchive::new(source)?;
    let mut summary = InspectSummary {
        source: args.source,
        ..InspectSummary::default()
    };

    for idx in 0..archive.len() {
        let file = archive.by_index(idx)?;
        let is_dir = file.name().ends_with('/');
        let entry = ArchiveEntry {
            name: file.name().to_string(),
            is_dir,
            compressed_size: file.compressed_size(),
            uncompressed_size: file.size(),
        };
        if is_dir {
            summary.dir_count += 1;
        } else {
            summary.file_count += 1;
        }
        summary.compressed_size += entry.compressed_size;
        summary.uncompressed_size += entry.uncompressed_size;
        summary.entries.push(entry);
    }

    Ok(summary)
}

pub fn print_inspect_summary(summary: &InspectSummary, quiet: bool) {
    if quiet {
        return;
    }

    println!("Archive: {}", summary.source);
    println!("Files: {}", summary.file_count);
    println!("Directories: {}", summary.dir_count);
    println!("Compressed bytes: {} B", summary.compressed_size);
    println!("Uncompressed bytes: {} B", summary.uncompressed_size);
    println!("Entries:");
    for entry in &summary.entries {
        let kind = if entry.is_dir { "dir" } else { "file" };
        println!("  [{kind}] {}", entry.name);
    }
}
