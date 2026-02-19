mod app;
mod config;
mod email;
mod template;

use app::EmailApp;
use config::SmtpConfig;
use template::load_templates;

fn main() {
    let smtp_config = match SmtpConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load Settings.toml: {}", e);
            eprintln!("Please ensure src/Settings.toml exists with host, port, username, password, from_name fields.");
            std::process::exit(1);
        }
    };

    let templates = load_templates();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Bulk Email Sender",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(EmailApp::new(smtp_config, templates)))
        }),
    );
}
