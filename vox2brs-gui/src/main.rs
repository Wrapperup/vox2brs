#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod app;

use std::env;
use app::Vox2BrsApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let build_dir = match env::consts::OS {
        "windows" => dirs::data_local_dir().unwrap().to_str().unwrap().to_string() + "\\Brickadia\\Saved\\Builds",
        "linux" => dirs::config_dir().unwrap().to_str().unwrap().to_string() + "/Epic/Brickadia/Saved/Builds",
        _ => String::new(),
    };

    let app = Vox2BrsApp {
        output_directory: build_dir,
        ..Default::default()
    };

    let native_options = eframe::NativeOptions {
        initial_window_size: Some([590.0, 400.0].into()),
        resizable: false,
        ..Default::default()
    };

    eframe::run_native(Box::new(app), native_options);
}