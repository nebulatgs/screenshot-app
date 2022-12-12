// use ffmpeg_next::{codec, encoder, format, log, media, Rational};

use anyhow::Context;
use std::process::{Child, Command, Stdio};
use tempdir::TempDir;

pub fn capture(x: u32, y: u32, w: u32, h: u32) -> anyhow::Result<(Child, TempDir)> {
    let dir = TempDir::new("screenshot-app")?;
    let file_path = dir.path().join("record.mp4");

    let cmd = Command::new("ffmpeg")
        .args([
            "-init_hw_device",
            "d3d11va",
            "-filter_complex",
            format!("ddagrab=0:framerate=60:output_fmt=auto:draw_mouse=0:video_size={w}x{h}:offset_x={x}:offset_y={y}").as_str(),
            "-c:v",
            "h264_nvenc",
            "-cq:v",
            "15",
            file_path.to_str().context("failed to convert path")?,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to spawn ffmpeg")?;

    Ok((cmd, dir))
}
