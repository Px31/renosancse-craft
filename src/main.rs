mod app;
mod audio;
mod config;
mod gui;
mod model;

use app::RenosancseApp;

fn main() -> iced::Result {
    env_logger::init();

    // Tiny CLI layer so bug reports can pin the exact version.
    let args: Vec<String> = std::env::args().skip(1).collect();
    for arg in &args {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--about" | "-a" => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                println!(
                    "FFmpeg: {}",
                    audio::probe::ffmpeg_version().unwrap_or_else(|| "not found on PATH".into())
                );
                return Ok(());
            }
            _ => {}
        }
    }

    // Embed the app icon so the window, taskbar, and Alt-Tab carry the brand.
    let icon = iced::window::icon::from_file_data(include_bytes!("../assets/icon.png"), None).ok();

    iced::application(
        "Renosancse Craft",
        RenosancseApp::update,
        RenosancseApp::view,
    )
    .window(iced::window::Settings {
        icon,
        ..iced::window::Settings::default()
    })
    .theme(RenosancseApp::theme)
    .run_with(RenosancseApp::new)
}
