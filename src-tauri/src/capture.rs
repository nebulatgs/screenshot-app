// use ffmpeg_next::{codec, encoder, format, log, media, Rational};

use std::os::windows::process::CommandExt;

use anyhow::Context;

use tauri::{
    api::process::{Command, CommandChild, CommandEvent},
    async_runtime::Receiver,
};
use tempdir::TempDir;
const CREATE_NO_WINDOW: u32 = 0x08000000;
pub fn capture(
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    monitor: u32,
) -> anyhow::Result<(Receiver<CommandEvent>, CommandChild, TempDir)> {
    let dir = TempDir::new("screenshot-app")?;
    let file_path = dir.path().join("record.mp4");
    let output = std::process::Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "name"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;
    let gpu = String::from_utf8_lossy(&output.stdout);

    if gpu.contains("NVIDIA") {
        let (rx, cmd) = Command::new_sidecar("ffmpeg")
        .expect("failed to find ffmpeg")
        .args([
            "-init_hw_device",
            "d3d11va",
            "-filter_complex",
            format!("ddagrab=0:output_idx={monitor}:framerate=60:output_fmt=auto:draw_mouse=0:video_size={w}x{h}:offset_x={x}:offset_y={y}").as_str(),
            "-c:v",
            "h264_nvenc",
            "-cq:v",
            "15",
            file_path.to_str().context("failed to convert path")?,
        ])
        .spawn()
        .context("failed to spawn ffmpeg")?;
        return Ok((rx, cmd, dir));
    } else {
        let (rx, cmd) = Command::new_sidecar("ffmpeg")
        .expect("failed to find ffmpeg")
        .args([
            "-init_hw_device",
            "d3d11va",
            "-filter_complex",
            format!("ddagrab=0:output_idx={monitor}:framerate=60:output_fmt=auto:draw_mouse=0:video_size={w}x{h}:offset_x={x}:offset_y={y},hwdownload,format=bgra").as_str(),
            "-c:v",
            "libx264",
            "-crf",
            "15",
            "-pix_fmt",
            "yuv420p",
            file_path.to_str().context("failed to convert path")?,
        ])
        .spawn()
        .context("failed to spawn ffmpeg")?;
        return Ok((rx, cmd, dir));
    }
}
