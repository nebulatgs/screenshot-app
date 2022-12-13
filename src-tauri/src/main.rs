#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

use anyhow::Context;
use rand::Rng;
use s3::{creds::Credentials, Bucket, Region};
use tauri::{
    api::process::{CommandChild, CommandEvent},
    async_runtime::Receiver,
    CustomMenuItem, GlobalShortcutManager, Manager, Position, State, SystemTray, SystemTrayEvent,
    SystemTrayMenu,
};
use tempdir::TempDir;

mod capture;

struct Storage {
    monitor: AtomicU32,
    ffmpeg_child: Mutex<Option<(Receiver<CommandEvent>, CommandChild, TempDir)>>,
    bucket: Mutex<Option<Bucket>>,
    custom_domain: Mutex<Option<String>>,
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
        capture::capture(
            (x1 as f64 * scale) as u32,
            (y1 as f64 * scale) as u32,
            w,
            h,
            storage.monitor.load(Ordering::Relaxed),
        )
        .unwrap(),
    );
}

#[tauri::command]
fn close_capture(app: tauri::AppHandle) {
    if let Some(window) = app.get_window("capture") {
        window.close().unwrap();
    }
}

#[tauri::command]
fn set_bucket(
    name: String,
    region: String,
    secret: String,
    access: String,
    endpoint: String,
    custom: String,
    storage: State<Storage>,
) {
    println!("set bucket");
    storage.bucket.lock().unwrap().replace(
        Bucket::new(
            &name,
            Region::Custom { region, endpoint },
            Credentials::new(Some(&access), Some(&secret), None, None, None).unwrap(),
        )
        .context("failed to create bucket")
        .unwrap(),
    );
    Option::replace(&mut storage.custom_domain.lock().unwrap(), custom);
}

fn main() {
    let tray_menu = SystemTrayMenu::new().add_item(CustomMenuItem::new("quit", "Quit App")); // insert the menu items here

    tauri::Builder::default()
        .setup(|app| {
            {
                let handle = app.handle();
                app.global_shortcut_manager()
                    .register("ALT + SHIFT + V", move || {
                        let window = tauri::WindowBuilder::new(
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
                        .visible(false)
                        .build()
                        .unwrap();
                        for (i, monitor) in window.available_monitors().unwrap().iter().enumerate()
                        {
                            use enigo::Enigo;
                            let (x, y) = Enigo::mouse_location();
                            let monitor_pos = monitor.position();
                            let monitor_size = monitor.size();
                            let monitor_x = monitor_pos.x as i32;
                            let monitor_y = monitor_pos.y as i32;
                            let monitor_w = monitor_size.width as i32;
                            let monitor_h = monitor_size.height as i32;

                            // check if mouse is in monitor
                            if x >= monitor_x
                                && x <= monitor_x + monitor_w
                                && y >= monitor_y
                                && y <= monitor_y + monitor_h
                            {
                                window
                                    .set_position(Position::Physical(*monitor.position()))
                                    .unwrap();
                                window.show().unwrap();
                                handle
                                    .state::<Storage>()
                                    .monitor
                                    .store(i as u32, Ordering::Relaxed);
                                break;
                            }
                        }
                    })
                    .unwrap();
            }
            {
                let handle = app.handle();
                app.global_shortcut_manager()
                    .register("ALT + SHIFT + S", move || {
                        if let Some((mut rx, mut child, temp_dir)) = handle
                            .state::<Storage>()
                            .ffmpeg_child
                            .lock()
                            .unwrap()
                            .take()
                        {
                            let handle = handle.clone();
                            tauri::async_runtime::spawn(async move {
                                // Ask ffmpeg to finish recording
                                child.write(b"q").unwrap();

                                // Hacky way to wait for ffmpeg to exit
                                while let Some(event) = rx.recv().await {
                                    if let CommandEvent::Stdout(line) = event {
                                        if line.contains("muxing overhead") {
                                            break;
                                        }
                                    }
                                }

                                handle.get_window("capture").unwrap().close().unwrap();
                                use chrono::Utc;
                                let now = Utc::now();
                                let filename =
                                    format!("qc_{}.mp4", now.format("%d-%m-%Y_%H-%M-%S"));
                                _ = std::fs::create_dir(
                                    dirs::video_dir().unwrap().join("Quick Captures"),
                                );
                                std::fs::copy(
                                    temp_dir.path().join("record.mp4"),
                                    dirs::video_dir()
                                        .unwrap()
                                        .join("Quick Captures")
                                        .join(&filename),
                                )
                                .unwrap();
                                let state = handle.state::<Storage>();
                                let guard = state.bucket.lock().unwrap().clone();
                                if guard.is_some() {
                                    let bucket = guard.unwrap();
                                    let mut file =
                                        tokio::fs::File::open(temp_dir.path().join("record.mp4"))
                                            .await
                                            .unwrap();
                                    let random_bytes = rand::thread_rng().gen::<[u8; 12]>();
                                    let filename = format!("/{}.mp4", &hex::encode(&random_bytes));
                                    bucket
                                        .put_object_stream_with_content_type(
                                            &mut file,
                                            &filename,
                                            "video/mp4",
                                        )
                                        .await
                                        .unwrap();
                                    if state.custom_domain.lock().unwrap().is_some() {
                                        let url = format!(
                                            "https://{}{}",
                                            state.custom_domain.lock().unwrap().as_ref().unwrap(),
                                            filename
                                        );
                                        use clipboard_win::{formats, set_clipboard};
                                        set_clipboard(formats::Unicode, &url).unwrap();

                                        println!("url: {}", url);
                                    }
                                }
                                temp_dir.close().unwrap();
                            });
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
            monitor: Default::default(),
            bucket: Default::default(),
            custom_domain: Default::default(),
        })
        .invoke_handler(tauri::generate_handler![capture, close_capture, set_bucket])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
