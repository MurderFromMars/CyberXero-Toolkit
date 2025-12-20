//! Multimedia tools page button handlers.
//!
//! Handles:
//! - OBS-Studio with plugins and V4L2
//! - Jellyfin server installation

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption,
};
use crate::ui::task_runner::{self, Command};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder};
use log::info;

/// Set up all button handlers for the multimedia tools page
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder) {
    setup_obs_studio_aio(page_builder);
    setup_jellyfin(page_builder);
}

fn setup_obs_studio_aio(page_builder: &Builder) {
    if let Some(btn_obs_studio_aio) = page_builder.object::<gtk4::Button>("btn_obs_studio_aio") {
        btn_obs_studio_aio.connect_clicked(move |button| {
            info!("Multimedia tools: OBS-Studio AiO button clicked");            let widget = button.clone().upcast::<gtk4::Widget>();
            let window = widget.root().and_then(|r| r.downcast::<ApplicationWindow>().ok());

            if let Some(window) = window {
                let window_clone = window.clone();
                let window_ref = window.upcast_ref::<gtk4::Window>();

                let obs_installed = core::is_flatpak_installed("com.obsproject.Studio");
                let wayland_hotkeys_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.WaylandHotkeys");
                let v4l2_installed = core::is_package_installed("v4l2loopback-dkms");

                let graphics_capture_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.OBSVkCapture") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.Gstreamer") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.GStreamerVaapi");

                let transitions_effects_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.MoveTransition") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.TransitionTable") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.ScaleToSound");

                let streaming_tools_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.WebSocket") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.SceneSwitcher") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.DroidCam");

                let audio_video_tools_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.waveform") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.VerticalCanvas") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.BackgroundRemoval");

                let config = SelectionDialogConfig::new(
                    "OBS-Studio AiO Installation",
                    "Select which components to install. All options are optional.",
                )
                .add_option(SelectionOption::new(
                    "obs",
                    "OBS-Studio",
                    "Main OBS-Studio application (Flatpak)",
                    obs_installed,
                ))
                .add_option(SelectionOption::new(
                    "wayland_hotkeys",
                    "Wayland Hotkeys Plugin",
                    "Enable hotkey support for OBS on Wayland",
                    wayland_hotkeys_installed,
                ))
                .add_option(SelectionOption::new(
                    "graphics_capture",
                    "Graphics Capture Plugins",
                    "VkCapture, GStreamer, GStreamer VA-API",
                    graphics_capture_installed,
                ))
                .add_option(SelectionOption::new(
                    "transitions_effects",
                    "Transitions & Effects",
                    "Move Transition, Transition Table, Scale to Sound",
                    transitions_effects_installed,
                ))
                .add_option(SelectionOption::new(
                    "streaming_tools",
                    "Streaming & Recording Tools",
                    "WebSocket API, Scene Switcher, DroidCam",
                    streaming_tools_installed,
                ))
                .add_option(SelectionOption::new(
                    "audio_video_tools",
                    "Audio & Video Tools",
                    "Waveform, Vertical Canvas, Background Removal",
                    audio_video_tools_installed,
                ))
                .add_option(SelectionOption::new(
                    "v4l2",
                    "V4L2loopback Virtual Camera",
                    "Enable OBS virtual camera functionality",
                    v4l2_installed,
                ))
                .confirm_label("Install");

                show_selection_dialog(window_ref, config, move |selected_ids| {
                    let mut commands: Vec<Command> = vec![];

                    if selected_ids.contains(&"obs".to_string()) {
                        commands.push(Command::normal(
                            "flatpak",
                            &["install", "-y", "com.obsproject.Studio"],
                            "Installing OBS-Studio...",
                        ));
                    }
                    if selected_ids.contains(&"wayland_hotkeys".to_string()) {
                        commands.push(Command::normal(
                            "flatpak",
                            &["install", "-y", "com.obsproject.Studio.Plugin.WaylandHotkeys"],
                            "Installing Wayland Hotkeys plugin...",
                        ));
                    }
                    if selected_ids.contains(&"graphics_capture".to_string()) {
                        commands.push(Command::normal(
                            "flatpak",
                            &[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.OBSVkCapture",
                                "org.freedesktop.Platform.VulkanLayer.OBSVkCapture/x86_64/25.08",
                                "com.obsproject.Studio.Plugin.Gstreamer",
                                "com.obsproject.Studio.Plugin.GStreamerVaapi",
                            ],
                            "Installing graphics capture plugins...",
                        ));
                    }
                    if selected_ids.contains(&"transitions_effects".to_string()) {
                        commands.push(Command::normal(
                            "flatpak",
                            &[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.MoveTransition",
                                "com.obsproject.Studio.Plugin.TransitionTable",
                                "com.obsproject.Studio.Plugin.ScaleToSound",
                            ],
                            "Installing transitions & effects plugins...",
                        ));
                    }
                    if selected_ids.contains(&"streaming_tools".to_string()) {
                        commands.push(Command::normal(
                            "flatpak",
                            &[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.WebSocket",
                                "com.obsproject.Studio.Plugin.SceneSwitcher",
                                "com.obsproject.Studio.Plugin.DroidCam",
                            ],
                            "Installing streaming tools...",
                        ));
                    }
                    if selected_ids.contains(&"audio_video_tools".to_string()) {
                        commands.push(Command::normal(
                            "flatpak",
                            &[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.waveform",
                                "com.obsproject.Studio.Plugin.VerticalCanvas",
                                "com.obsproject.Studio.Plugin.BackgroundRemoval",
                            ],
                            "Installing audio/video enhancement plugins...",
                        ));
                    }
                    if selected_ids.contains(&"v4l2".to_string()) {
                        commands.push(Command::aur(
                            &["-S", "--noconfirm", "--needed", "v4l2loopback-dkms", "v4l2loopback-utils"],
                            "Installing V4L2 loopback modules...",
                        ));
                        commands.push(Command::privileged(
                            "sh",
                            &["-c", "echo 'v4l2loopback' > /etc/modules-load.d/v4l2loopback.conf"],
                            "Enabling V4L2 loopback module at boot...",
                        ));
                        commands.push(Command::privileged(
                            "sh",
                            &[
                                "-c",
                                "echo 'options v4l2loopback exclusive_caps=1 card_label=\"OBS Virtual Camera\"' > /etc/modprobe.d/v4l2loopback.conf",
                            ],
                            "Configuring virtual camera options...",
                        ));
                    }

                    if !commands.is_empty() {
                        let window_ref2 = window_clone.upcast_ref::<gtk4::Window>();
                        task_runner::run(
                            window_ref2,
                            commands,
                            "OBS-Studio Setup",
                            None,
                        );
                    }
                });
            }
        });
    }
}

fn setup_jellyfin(page_builder: &Builder) {
    if let Some(btn_jellyfin) = page_builder.object::<gtk4::Button>("btn_jellyfin") {
        btn_jellyfin.connect_clicked(move |button| {
            info!("Multimedia tools: Jellyfin button clicked");
            let commands = vec![
                Command::aur(
                    &[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "jellyfin-server",
                        "jellyfin-web",
                        "jellyfin-ffmpeg",
                    ],
                    "Installing Jellyfin server and components...",
                ),
                Command::privileged(
                    "systemctl",
                    &["enable", "--now", "jellyfin.service"],
                    "Starting Jellyfin service...",
                ),
            ];

            let widget = button.clone().upcast::<gtk4::Widget>();
            let window = widget
                .root()
                .and_then(|r| r.downcast::<ApplicationWindow>().ok());
            if let Some(window) = window {
                let window_ref = window.upcast_ref::<gtk4::Window>();
                task_runner::run(window_ref, commands, "Jellyfin Server Setup", None);
            }
        });
    }
}
