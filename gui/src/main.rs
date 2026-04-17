//! CyberXero Toolkit - System management and customization application.

use adw::prelude::*;
use adw::Application;
use log::info;

mod config;
mod core;
mod ui;

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    info!(
        "Starting {} v{}",
        config::app_info::NAME,
        config::app_info::VERSION
    );
    info!("Application ID: {}", config::app_info::ID);

    let app = Application::builder()
        .application_id(config::app_info::ID)
        .build();

    app.connect_activate(ui::setup_application_ui);

    app.run();
}
