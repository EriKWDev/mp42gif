use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Convert an MP4 to an optimized GIF using palette generation.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Input MP4 file
    input: PathBuf,

    /// Output GIF file
    output: PathBuf,

    /// Frames per second
    #[arg(long, default_value = "15")]
    fps: u32,

    /// Scale factor (e.g., 0.5 for half size)
    #[arg(long, default_value = "0.4")]
    scale: f32,
}

fn run_ffmpeg(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err("ffmpeg failed".into());
    }

    Ok(())
}

fn run_ffprobe(args: &[&str]) {
    let _ = Command::new("ffprobe")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let palette = std::env::temp_dir().join("palette.png");
    let fps_filter = format!("fps={}", args.fps);
    let scale_filter = format!("scale=iw*{}:ih*{}:flags=lanczos", args.scale, args.scale);

    println!("Input info:");
    run_ffprobe(&[
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-show_streams",
        "-pretty",
        "-i",
        args.input.to_str().unwrap(),
    ]);

    println!("\nGenerating palette...");
    run_ffmpeg(&[
        "-i",
        args.input.to_str().unwrap(),
        "-vf",
        &format!("{},{},palettegen", fps_filter, scale_filter),
        "-y",
        palette.to_str().unwrap(),
    ])?;

    println!("\nGenerating GIF...");
    run_ffmpeg(&[
        "-i",
        args.input.to_str().unwrap(),
        "-i",
        palette.to_str().unwrap(),
        "-filter_complex",
        &format!("[0:v]{},{}[x];[x][1:v]paletteuse", fps_filter, scale_filter),
        "-y",
        args.output.to_str().unwrap(),
    ])?;

    std::fs::remove_file(&palette).ok();
    println!("\nDone.");
    Ok(())
}
