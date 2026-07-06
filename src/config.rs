use crate::error::ZipperResult;
use crate::pack::{PackRequest, PackRuntimeOptions};
use crate::{CompressionMode, OverwriteMode};
use serde_json::Value;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

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

pub fn resolve_pack_options(raw: PackRequest) -> ZipperResult<PackRuntimeOptions> {
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

fn load_pack_config(config_path: Option<&str>) -> ZipperResult<PackConfig> {
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

fn read_pack_config_file(path: &Path) -> ZipperResult<Option<PackConfig>> {
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;

    if content.trim().is_empty() {
        return Ok(Some(PackConfig::default()));
    }

    let value: Value = serde_json::from_str(&content)?;

    let config_obj = if path.file_name().and_then(|name| name.to_str()) == Some("package.json") {
        value.get("rz-zip").cloned()
    } else {
        Some(value)
    };

    let Some(config_obj) = config_obj else {
        return Ok(None);
    };

    Ok(Some(parse_pack_config(&config_obj)?))
}

fn parse_pack_config(value: &Value) -> ZipperResult<PackConfig> {
    let as_obj = value.as_object().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Config root must be a JSON object",
        )
    })?;

    Ok(PackConfig {
        source: as_obj
            .get("source")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        output: as_obj
            .get("output")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        format: as_obj
            .get("format")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        compression: as_obj
            .get("compression")
            .and_then(|v| v.as_str())
            .map(|v| parse_compression_mode("compression", v))
            .transpose()?,
        level: as_obj.get("level").map(parse_level).transpose()?,
        output_dir: as_obj
            .get("output_dir")
            .or_else(|| as_obj.get("output-dir"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        excludes: parse_string_array(as_obj.get("excludes").or_else(|| as_obj.get("exclude"))),
        includes: parse_string_array(as_obj.get("includes").or_else(|| as_obj.get("include"))),
        sha256: as_obj
            .get("sha256")
            .map(|v| parse_bool("sha256", v))
            .transpose()?,
        base_dir: as_obj
            .get("base_dir")
            .or_else(|| as_obj.get("base-dir"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        strip_prefix: as_obj
            .get("strip_prefix")
            .or_else(|| as_obj.get("strip-prefix"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        no_prefix: as_obj
            .get("no_prefix")
            .map(|v| parse_bool("no_prefix", v))
            .transpose()?,
        overwrite: as_obj
            .get("overwrite")
            .and_then(|v| v.as_str())
            .map(|v| parse_overwrite_mode("overwrite", v))
            .transpose()?,
        dry_run: as_obj
            .get("dry_run")
            .map(|v| parse_bool("dry_run", v))
            .transpose()?,
        quiet: as_obj
            .get("quiet")
            .map(|v| parse_bool("quiet", v))
            .transpose()?,
        verbose: as_obj
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

fn parse_compression_mode(key: &str, value: &str) -> ZipperResult<CompressionMode> {
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

fn parse_overwrite_mode(key: &str, value: &str) -> ZipperResult<OverwriteMode> {
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

fn parse_level(value: &Value) -> ZipperResult<u8> {
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

fn parse_bool(key: &str, value: &Value) -> ZipperResult<bool> {
    value.as_bool().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("`{key}` must be a boolean value"),
        )
        .into()
    })
}
