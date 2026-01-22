use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, ValueHint};
use env_logger::{Builder, Env};

use image_processor::{load_image_as_rgba, plugin_loader::Plugin, save_rgba_as_png};

#[derive(Parser, Debug)]
struct Args {
    /// Path to input PNG image
    #[arg(long, value_hint = ValueHint::FilePath)]
    input: PathBuf,

    /// Path for output PNG image
    #[arg(long, value_hint = ValueHint::FilePath)]
    output: PathBuf,

    /// Plugin name (without extension)
    #[arg(long)]
    plugin: String,

    /// Path to parameters file(json)
    #[arg(long, value_hint = ValueHint::FilePath)]
    params: PathBuf,

    /// Path to plugin directory
    #[arg(short = 'd', long = "plugin-path", default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    init_logger();

    let args = Args::parse();

    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {:?}", args.input);
    }

    if !args.params.exists() {
        anyhow::bail!("Parameters file does not exist: {:?}", args.params);
    }

    if !args.plugin_path.exists() {
        anyhow::bail!("Plugin path does not exist: {:?}", args.plugin_path);
    }

    let params = std::fs::read_to_string(&args.params)
        .with_context(|| format!("Failed to read parameters file: {:?}", args.params))?;

    log::info!("Using parameters: {}", params.trim());

    let (width, height, mut rgba_data) = load_image_as_rgba(&args.input)
        .with_context(|| format!("Failed to load image: {:?}", args.input))?;

    log::info!(
        "Image loaded: {}x{} ({} bytes)",
        width,
        height,
        rgba_data.len()
    );

    let plugin = Plugin::load(&args.plugin_path, &args.plugin).with_context(|| {
        format!(
            "Failed to load plugin '{}' from {:?}",
            args.plugin, args.plugin_path
        )
    })?;

    let interface = plugin.inteface()?;

    log::info!("Plugin loaded successfully");

    let code = (interface.process_image)(
        width,
        height,
        rgba_data.as_mut_ptr(),
        std::ffi::CString::new(params)?.as_ptr(),
    );

    match code {
        plugin_lib::OK_CODE => {
            log::info!("Processing completed successfully.",);
        }
        plugin_lib::PARSE_ERROR_CODE => {
            anyhow::bail!("Failed to parse parameters");
        }

        plugin_lib::DATA_IMAGE_ERROR_CODE => {
            anyhow::bail!("Data image error");
        }

        plugin_lib::INVALID_PARAMS_CODE => {
            anyhow::bail!("Invalid parameters");
        }

        _ => {
            anyhow::bail!("Processing failed with code: {}.", code);
        }
    }

    log::info!("Image processed by plugin");

    save_rgba_as_png(&args.output, width, height, &rgba_data)
        .with_context(|| format!("Failed to save image: {:?}", args.output))?;

    log::info!("Comlite");
    log::info!("Image saved to: {:?}", args.output);

    Ok(())
}

fn init_logger() {
    // TODO: Добавить логи из модулей
    Builder::from_env(Env::default().default_filter_or("info")).init();
}
