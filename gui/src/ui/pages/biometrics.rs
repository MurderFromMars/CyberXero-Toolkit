//! Biometrics page button handlers.
//!
//! Handles:
//! - Fingerprint reader setup (xfprintd-gui)
//! - Howdy facial recognition setup (xero-howdy-qt)

use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder};
use log::info;

/// Set up all button handlers for the biometrics page
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_fingerprint(page_builder, window);
    setup_howdy(page_builder, window);
}

fn setup_fingerprint(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fingerprint_setup =
        extract_widget::<gtk4::Button>(page_builder, "btn_fingerprint_setup");
    let window = window.clone();
    btn_fingerprint_setup.connect_clicked(move |_| {
        info!("Biometrics: Fingerprint setup button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "xfprintd-gui"])
                    .description("Installing Fingerprint GUI Tool...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Install Fingerprint GUI Tool",
        );
    });
}

fn setup_howdy(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_howdy_setup = extract_widget::<gtk4::Button>(page_builder, "btn_howdy_setup");
    let window = window.clone();
    btn_howdy_setup.connect_clicked(move |_| {
        info!("Biometrics: Howdy setup button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "xero-howdy-qt"])
                    .description("Installing Xero Howdy Qt...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install Xero Howdy Qt");
    });
}
