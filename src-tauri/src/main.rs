#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{io::Write, process::Child, sync::Mutex};

use anyhow::Context;
use tauri::{
    CustomMenuItem, GlobalShortcutManager, Manager, State, SystemTray, SystemTrayEvent,
    SystemTrayMenu,
};
use tempdir::TempDir;

mod capture;

struct Storage {
    ffmpeg_child: Mutex<Option<(Child, TempDir)>>,
}

#[tauri::command]
fn capture(x1: u32, y1: u32, x2: u32, y2: u32, storage: State<Storage>, app: tauri::AppHandle) {
    let window = app.get_window("capture").unwrap();
    window.set_ignore_cursor_events(true).unwrap();
    let scale = window
        .current_monitor()
        .unwrap()
        .context("failed to get current monitor")
        .unwrap()
        .scale_factor();
    let w = (x2 as f64 * scale) as u32 - (x1 as f64 * scale) as u32;
    let h = (y2 as f64 * scale) as u32 - (y1 as f64 * scale) as u32;
    storage.ffmpeg_child.lock().unwrap().replace(
        capture::capture((x1 as f64 * scale) as u32, (y1 as f64 * scale) as u32, w, h).unwrap(),
    );
}

#[tauri::command]
fn close_capture(app: tauri::AppHandle) {
    if let Some(window) = app.get_window("capture") {
        window.close().unwrap();
    }
}
fn main() {
    let tray_menu = SystemTrayMenu::new().add_item(CustomMenuItem::new("quit", "Quit App")); // insert the menu items here

    tauri::Builder::default()
        .setup(|app| {
            {
                let handle = app.handle();
                app.global_shortcut_manager()
                    .register("ALT + SHIFT + V", move || {
                        tauri::WindowBuilder::new(
                            &handle,
                            "capture",
                            tauri::WindowUrl::App("/capture".into()),
                        )
                        .transparent(true)
                        .focused(true)
                        .decorations(false)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .fullscreen(true)
                        .resizable(false)
                        .build()
                        .unwrap();
                    })
                    .unwrap();
            }
            {
                let handle = app.handle();
                app.global_shortcut_manager()
                    .register("ALT + SHIFT + S", move || {
                        if let Some((mut child, temp_dir)) = handle
                            .state::<Storage>()
                            .ffmpeg_child
                            .lock()
                            .unwrap()
                            .take()
                        {
                            child.stdin.take().unwrap().write_all(b"q").unwrap();
                            handle.get_window("capture").unwrap().close().unwrap();
                            child.wait().unwrap();
                            use chrono::Utc;
                            let now = Utc::now();
                            let filename = format!("qc_{}.mp4", now.format("%d-%m-%Y_%H-%M-%S"));
                            _ = std::fs::create_dir(
                                dirs::video_dir().unwrap().join("Quick Captures"),
                            );
                            std::fs::copy(
                                temp_dir.path().join("record.mp4"),
                                dirs::video_dir()
                                    .unwrap()
                                    .join("Quick Captures")
                                    .join(filename),
                            )
                            .unwrap();
                            temp_dir.close().unwrap();
                        }
                        // capture_window
                        //     .set_position(Position::Physical(PhysicalPosition { x: 0, y: 0 }))
                        //     .unwrap();
                        // capture::capture().unwrap();
                    })
                    .unwrap();
            }

            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|_, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a left click");
            }
            SystemTrayEvent::RightClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a right click");
            }
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a double click");
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .manage(Storage {
            ffmpeg_child: Default::default(),
        })
        .invoke_handler(tauri::generate_handler![capture, close_capture])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
