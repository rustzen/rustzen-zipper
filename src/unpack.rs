use crate::error::ZipperResult;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self};
use std::path::Path;
use zip::read::ZipArchive;

#[derive(Clone, Debug)]
pub struct UnpackOptions {
    pub source: String,
    pub output_dir: String,
    pub quiet: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UnpackSummary {
    pub source: String,
    pub output_dir: String,
    pub extracted_entries: usize,
}

pub fn unpack_archive(args: UnpackOptions) -> ZipperResult<UnpackSummary> {
    let source_path = Path::new(&args.source);
    if !source_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source zip does not exist: {}", args.source),
        )
        .into());
    }

    let output_root = Path::new(&args.output_dir);
    fs::create_dir_all(output_root)?;

    let source = File::open(source_path)?;
    let mut archive = ZipArchive::new(source)?;
    let mut extracted_entries = 0;

    for idx in 0..archive.len() {
        let mut file = archive.by_index(idx)?;
        let out_name = file
            .enclosed_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid zip entry name"))?;
        let out_path = output_root.join(out_name);

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
            extracted_entries += 1;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output = File::create(&out_path)?;
        io::copy(&mut file, &mut output)?;
        extracted_entries += 1;

        if !args.quiet {
            println!("  extracted: {}", out_path.display());
        }
    }

    if !args.quiet {
        println!("Unpacked: {} entry(s)", extracted_entries);
    }

    Ok(UnpackSummary {
        source: args.source,
        output_dir: args.output_dir,
        extracted_entries,
    })
}
