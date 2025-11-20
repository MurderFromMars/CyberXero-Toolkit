//! Multimedia tools page button handlers.
//!
//! Handles:
//! - OBS-Studio with plugins and V4L2
//! - Jellyfin server installation

use crate::core;
use crate::ui::selection_dialog;
use crate::ui::terminal;
use crate::utils;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Label};
use log::{info, warn};
use vte4::prelude::*;
use vte4::Terminal;

/// Set up all button handlers for the multimedia tools page
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    setup_obs_studio_aio(&page_builder, &terminal_output, &terminal_title);
    setup_jellyfin(&page_builder, &terminal_output, &terminal_title);
}

fn setup_obs_studio_aio(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_obs_studio_aio) = page_builder.object::<gtk4::Button>("btn_obs_studio_aio") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_obs_studio_aio.connect_clicked(move |button| {
            info!("Multimedia tools: OBS-Studio AiO button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let widget = button.clone().upcast::<gtk4::Widget>();
            let window = widget.root()
                .and_then(|root| root.downcast::<ApplicationWindow>().ok());

            if let Some(window) = window {
                let terminal_for_dialog = terminal_clone.clone();
                let title_for_dialog = title_clone.clone();
                let button_for_dialog = button.clone();
                let window_ref = window.upcast_ref::<gtk4::Window>();

                let obs_installed = core::is_flatpak_installed("com.obsproject.Studio");
                let v4l2_installed = core::is_package_installed("v4l2loopback-dkms");

                let config = selection_dialog::SelectionDialogConfig::new(
                    "OBS-Studio AiO Installation",
                    "Select which components to install. All options are optional.",
                )
                .add_option(selection_dialog::SelectionOption::new(
                    "obs",
                    "OBS-Studio",
                    "Main OBS-Studio application (Flatpak)",
                    obs_installed,
                ))
                .add_option(selection_dialog::SelectionOption::new(
                    "graphics_capture",
                    "Graphics Capture Plugins",
                    "VkCapture, GStreamer, GStreamer VA-API (for game/screen capture)",
                    false,
                ))
                .add_option(selection_dialog::SelectionOption::new(
                    "transitions_effects",
                    "Transitions & Effects",
                    "Move Transition, Transition Table, Scale to Sound (visual effects)",
                    false,
                ))
                .add_option(selection_dialog::SelectionOption::new(
                    "streaming_tools",
                    "Streaming & Recording Tools",
                    "WebSocket API, Scene Switcher, DroidCam (advanced features)",
                    false,
                ))
                .add_option(selection_dialog::SelectionOption::new(
                    "audio_video_tools",
                    "Audio & Video Tools",
                    "Waveform, Vertical Canvas, Background Removal (content tools)",
                    false,
                ))
                .add_option(selection_dialog::SelectionOption::new(
                    "v4l2",
                    "V4L2loopback Virtual Camera",
                    "Enable OBS virtual camera functionality (requires reboot)",
                    v4l2_installed,
                ))
                .confirm_label("Install");

                selection_dialog::show_selection_dialog(
                    window_ref,
                    config,
                    move |selected_ids| {
                        if selected_ids.is_empty() {
                            return;
                        }

                        let helper = match utils::detect_aur_helper() {
                            Some(h) => h,
                            None => {
                                warn!("No AUR helper detected");
                                terminal_for_dialog.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                                return;
                            }
                        };

                        let mut commands = vec![];

                        if selected_ids.contains(&"obs".to_string()) {
                            commands.push(terminal::TerminalCommand::new("flatpak",
                                &["install", "-y", "com.obsproject.Studio"]));
                        }

                        if selected_ids.contains(&"graphics_capture".to_string()) {
                            commands.push(terminal::TerminalCommand::new("flatpak",
                                &["install", "-y",
                                  "com.obsproject.Studio.Plugin.OBSVkCapture",
                                  "org.freedesktop.Platform.VulkanLayer.OBSVkCapture",
                                  "com.obsproject.Studio.Plugin.Gstreamer",
                                  "com.obsproject.Studio.Plugin.GStreamerVaapi"]));
                        }

                        if selected_ids.contains(&"transitions_effects".to_string()) {
                            commands.push(terminal::TerminalCommand::new("flatpak",
                                &["install", "-y",
                                  "com.obsproject.Studio.Plugin.MoveTransition",
                                  "com.obsproject.Studio.Plugin.TransitionTable",
                                  "com.obsproject.Studio.Plugin.ScaleToSound"]));
                        }

                        if selected_ids.contains(&"streaming_tools".to_string()) {
                            commands.push(terminal::TerminalCommand::new("flatpak",
                                &["install", "-y",
                                  "com.obsproject.Studio.Plugin.WebSocket",
                                  "com.obsproject.Studio.Plugin.SceneSwitcher",
                                  "com.obsproject.Studio.Plugin.DroidCam"]));
                        }

                        if selected_ids.contains(&"audio_video_tools".to_string()) {
                            commands.push(terminal::TerminalCommand::new("flatpak",
                                &["install", "-y",
                                  "com.obsproject.Studio.Plugin.waveform",
                                  "com.obsproject.Studio.Plugin.VerticalCanvas",
                                  "com.obsproject.Studio.Plugin.BackgroundRemoval"]));
                        }

                        if selected_ids.contains(&"v4l2".to_string()) {
                            commands.push(terminal::TerminalCommand::new(helper,
                                &["-S", "--noconfirm", "--needed", "v4l2loopback-dkms", "v4l2loopback-utils"]));
                            commands.push(terminal::TerminalCommand::new("sudo", &["sh", "-c",
                                "echo 'v4l2loopback' > /etc/modules-load.d/v4l2loopback.conf"]));
                            commands.push(terminal::TerminalCommand::new("sudo", &["sh", "-c",
                                "echo 'options v4l2loopback exclusive_caps=1 card_label=\"OBS Virtual Camera\"' > /etc/modprobe.d/v4l2loopback.conf"]));
                        }

                        if !commands.is_empty() {
                            terminal::run_terminal_commands(
                                &button_for_dialog,
                                &terminal_for_dialog,
                                &title_for_dialog,
                                commands,
                                "OBS-Studio Setup",
                            );
                        }
                    },
                );
            }
        });
    }
}

fn setup_jellyfin(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_jellyfin) = page_builder.object::<gtk4::Button>("btn_jellyfin") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_jellyfin.connect_clicked(move |button| {
            info!("Multimedia tools: Jellyfin button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let helper = match utils::detect_aur_helper() {
                Some(h) => h,
                None => {
                    warn!("No AUR helper detected");
                    terminal_clone.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                    return;
                }
            };

            let commands = vec![
                terminal::TerminalCommand::new(helper,
                    &["-S", "--noconfirm", "--needed",
                      "jellyfin-server", "jellyfin-web", "jellyfin-ffmpeg"]),
                terminal::TerminalCommand::new("sudo",
                    &["systemctl", "enable", "--now", "jellyfin.service"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Jellyfin Server Installation",
            );
        });
    }
}
