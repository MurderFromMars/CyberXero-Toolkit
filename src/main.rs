use gtk4::Application;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use log::info;

mod config;
mod core;
mod ui;
mod utils;

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
