use rustzen_zipper::inspect::{inspect_archive, InspectOptions, InspectSummary};
use rustzen_zipper::unpack::{unpack_archive, UnpackOptions, UnpackSummary};

#[tauri::command]
fn inspect_zip(source: String) -> Result<InspectSummary, String> {
    inspect_archive(InspectOptions {
        source,
        quiet: true,
        json: false,
    })
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn unpack_zip(source: String, output_dir: String) -> Result<UnpackSummary, String> {
    unpack_archive(UnpackOptions {
        source,
        output_dir,
        quiet: true,
    })
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![inspect_zip, unpack_zip, open_path])
        .run(tauri::generate_context!())
        .expect("failed to run Rustzen Zipper macOS app");
}
