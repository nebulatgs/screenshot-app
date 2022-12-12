#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{io::Write, process::Child, sync::Mutex};

use anyhow::Context;
use tauri::{
    GlobalShortcutManager, Manager, PhysicalPosition, Position, State, SystemTray, SystemTrayEvent,
    SystemTrayMenu,
};
use tempdir::TempDir;

mod capture;

struct Storage {
    ffmpeg_child: Mutex<Option<(Child, TempDir)>>,
}
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
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

// async fn minimize(app_handle: tauri::AppHandle) {
//     use tauri::GlobalShortcutManager;
//     app_handle
//         .global_shortcut_manager()
//         .register("CTRL + U", move || {});
// }

fn main() {
    let tray_menu = SystemTrayMenu::new(); // insert the menu items here

    tauri::Builder::default()
        .setup(|app| {
            // let main_window = app.get_window("main").unwrap();
            // main_window.set_always_on_top(true).unwrap();
            // main_window.set_decorations(false).unwrap();
            // main_window
            //     .set_position(Position::Physical(PhysicalPosition { x: 0, y: 0 }))
            //     .unwrap();
            {
                let handle = app.handle();
                app.global_shortcut_manager()
                    .register("ALT + SHIFT + V", move || {
                        let capture_window = tauri::WindowBuilder::new(
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
                        // capture_window
                        //     .set_position(Position::Physical(PhysicalPosition { x: 0, y: 0 }))
                        //     .unwrap();
                        // capture::capture().unwrap();
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
        .on_system_tray_event(|app, event| match event {
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
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
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
