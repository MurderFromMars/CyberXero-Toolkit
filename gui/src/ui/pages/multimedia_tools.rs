//! Multimedia tools page button handlers.
//!
//! Handles:
//! - OBS-Studio with plugins and V4L2
//! - Kdenlive video editor
//! - Jellyfin server installation
//! - GPU Screen Recorder GTK (repo-first, AUR fallback)
//! - Streaming service web app installer

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder};
use log::info;

/// Streaming service entries: (name, url)
const STREAMING_SERVICES: &[(&str, &str)] = &[
    ("ABC IView", "https://iview.abc.net.au"),
    ("AirGPU", "https://app.airgpu.com"),
    ("Amazon Luna", "https://luna.amazon.com/"),
    ("Amazon Prime Video", "https://www.amazon.com/video"),
    ("Angry Birds TV", "https://www.angrybirds.com/series/"),
    ("Antstream", "https://live.antstream.com/"),
    ("Apple TV", "https://tv.apple.com/"),
    ("BBC iPlayer", "https://www.bbc.co.uk/iplayer/"),
    ("BritBox", "https://britbox.com"),
    ("Binge", "https://binge.com.au"),
    ("Blacknut", "https://www.blacknut.com/en-gb/games"),
    ("Boosteroid", "https://cloud.boosteroid.com"),
    ("CBBC", "https://www.bbc.co.uk/cbbc"),
    ("CBeebies", "https://www.bbc.co.uk/cbeebies"),
    ("Channel 4", "https://www.channel4.com/"),
    ("Crave", "https://www.crave.ca/"),
    ("Criterion Channel", "https://www.criterionchannel.com"),
    ("Crunchyroll", "https://www.crunchyroll.com/"),
    ("Curiosity Stream", "https://curiositystream.com"),
    ("Daily Wire", "https://www.dailywire.com/watch"),
    ("Discord", "https://discord.com/app"),
    ("Disney+", "https://www.disneyplus.com/"),
    ("DocPlay", "https://www.docplay.com"),
    ("Dropout", "https://www.dropout.tv/browse"),
    ("Emby Theater", "https://emby.media/"),
    ("Fox", "https://www.fox.com/"),
    ("Fubo TV", "https://www.fubo.tv"),
    ("GeForce Now", "https://play.geforcenow.com/mall/"),
    ("GBNews Live", "https://www.gbnews.com/watch/live"),
    ("GlobalComix", "https://globalcomix.com/"),
    ("Google Play Books", "https://play.google.com/store/books"),
    ("HBO Max", "https://www.max.com/"),
    ("Home Assistant", "https://demo.home-assistant.io/"),
    ("Hulu", "https://www.hulu.com/"),
    ("Internet Archive Movies", "https://archive.org/details/movies"),
    ("ITV X", "https://www.itv.com/"),
    ("Kanopy", "https://www.kanopy.com"),
    ("Microsoft Movies and TV", "https://apps.microsoft.com/movies"),
    ("My5", "https://www.channel5.com/"),
    ("Nebula", "https://nebula.tv/"),
    ("Netflix", "https://www.netflix.com/"),
    ("Newgrounds Movies", "https://www.newgrounds.com/movies"),
    ("Newgrounds Games", "https://www.newgrounds.com/games"),
    ("Kogama", "https://www.kogama.com/"),
    ("Paramount+", "https://www.paramountplus.com/"),
    ("Peacock TV", "https://www.peacocktv.com/"),
    ("POP Player", "https://player.pop.co.uk/"),
    ("Puffer", "https://puffer.stanford.edu/player/"),
    ("Plex", "https://app.plex.tv/"),
    ("Pocket Casts", "https://play.pocketcasts.com"),
    ("Poki", "https://poki.com/"),
    ("Reddit", "https://www.reddit.com/r/all/"),
    ("SBS Ondemand", "https://www.sbs.com.au/ondemand/"),
    ("Scratch", "https://scratch.mit.edu/explore/projects/all"),
    ("Sling TV", "https://www.sling.com"),
    ("Spotify", "https://open.spotify.com/"),
    ("Stan", "https://www.stan.com.au"),
    ("Steam Broadcasts", "https://steamcommunity.com/?subsection=broadcasts"),
    ("Squid TV", "https://www.squidtv.net/"),
    ("TikTok", "https://www.tiktok.com/"),
    ("Threads", "https://www.threads.net/"),
    ("Twitch", "https://www.twitch.tv/"),
    ("Twitter", "https://twitter.com/"),
    ("Vimeo", "https://vimeo.com/"),
    ("Virgin TV Go", "https://virgintvgo.virginmedia.com/en/home"),
    ("VK Play", "https://cloud.vkplay.ru/"),
    ("Xbox Game Pass Streaming", "https://www.xbox.com/play"),
    ("Xiaohongshu (RedNote)", "https://www.xiaohongshu.com/explore"),
    ("YouTube Music", "https://music.youtube.com/"),
    ("YouTube TV", "https://tv.youtube.com/"),
    ("YouTube", "https://www.youtube.com/"),
    ("WebRcade", "https://play.webrcade.com/"),
];

/// Set up all button handlers for the multimedia tools page
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_obs_studio_aio(page_builder, window);
    setup_kdenlive(page_builder, window);
    setup_jellyfin(page_builder, window);
    setup_gpu_screen_recorder(page_builder, window);
    setup_streaming_services(page_builder, window);
}

fn setup_obs_studio_aio(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_obs_studio_aio = extract_widget::<gtk4::Button>(page_builder, "btn_obs_studio_aio");
    let window = window.clone();
    btn_obs_studio_aio.connect_clicked(move |_| {
        info!("Multimedia tools: OBS-Studio AiO button clicked");
        let window_ref = window.upcast_ref();

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
                    "OBS-Studio & Plugins Installation",
                    "OBS-Studio will be installed. Optionally select plugins to install.",
                )
                .selection_type(SelectionType::Multi)
                .selection_required(false)
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

                let window_for_closure = window.clone();
                show_selection_dialog(window_ref, config, move |selected_ids| {
                    let mut commands = CommandSequence::new();

                    // Always install OBS-Studio
                    commands = commands.then(Command::builder()
                        .normal()
                        .program("flatpak")
                        .args(&["install", "-y", "com.obsproject.Studio"])
                        .description("Installing OBS-Studio...")
                        .build());

                    if selected_ids.iter().any(|s| s == "wayland_hotkeys") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&["install", "-y", "com.obsproject.Studio.Plugin.WaylandHotkeys"])
                            .description("Installing Wayland Hotkeys plugin...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "graphics_capture") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.OBSVkCapture",
                                "org.freedesktop.Platform.VulkanLayer.OBSVkCapture/x86_64/25.08",
                                "com.obsproject.Studio.Plugin.Gstreamer",
                                "com.obsproject.Studio.Plugin.GStreamerVaapi",
                            ])
                            .description("Installing graphics capture plugins...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "transitions_effects") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.MoveTransition",
                                "com.obsproject.Studio.Plugin.TransitionTable",
                                "com.obsproject.Studio.Plugin.ScaleToSound",
                            ])
                            .description("Installing transitions & effects plugins...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "streaming_tools") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.WebSocket",
                                "com.obsproject.Studio.Plugin.SceneSwitcher",
                                "com.obsproject.Studio.Plugin.DroidCam",
                            ])
                            .description("Installing streaming tools...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "audio_video_tools") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.waveform",
                                "com.obsproject.Studio.Plugin.VerticalCanvas",
                                "com.obsproject.Studio.Plugin.BackgroundRemoval",
                            ])
                            .description("Installing audio/video enhancement plugins...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "v4l2") {
                        commands = commands.then(Command::builder()
                            .aur()
                            .args(&["-S", "--noconfirm", "--needed", "v4l2loopback-dkms", "v4l2loopback-utils"])
                            .description("Installing V4L2 loopback modules...")
                            .build());
                        commands = commands.then(Command::builder()
                            .privileged()
                            .program("sh")
                            .args(&["-c", "echo 'v4l2loopback' > /etc/modules-load.d/v4l2loopback.conf"])
                            .description("Enabling V4L2 loopback module at boot...")
                            .build());
                        commands = commands.then(Command::builder()
                            .privileged()
                            .program("sh")
                            .args(&[
                                "-c",
                                "echo 'options v4l2loopback exclusive_caps=1 card_label=\"OBS Virtual Camera\"' > /etc/modprobe.d/v4l2loopback.conf",
                            ])
                            .description("Configuring virtual camera options...")
                            .build());
                    }

                    task_runner::run(window_for_closure.upcast_ref(), commands.build(), "OBS-Studio Setup");
                });
    });
}

fn setup_kdenlive(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_kdenlive = extract_widget::<gtk4::Button>(page_builder, "btn_kdenlive");
    let window = window.clone();
    btn_kdenlive.connect_clicked(move |_| {
        info!("Multimedia tools: Kdenlive button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "kdenlive"])
                    .description("Installing Kdenlive...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Kdenlive Installation");
    });
}

fn setup_jellyfin(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_jellyfin = extract_widget::<gtk4::Button>(page_builder, "btn_jellyfin");
    let window = window.clone();
    btn_jellyfin.connect_clicked(move |_| {
        info!("Multimedia tools: Jellyfin button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "jellyfin-server",
                        "jellyfin-web",
                        "jellyfin-ffmpeg",
                    ])
                    .description("Installing Jellyfin server and components...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "jellyfin.service"])
                    .description("Starting Jellyfin service...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Jellyfin Server Setup");
    });
}

fn setup_gpu_screen_recorder(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_gpu_screen_recorder =
        extract_widget::<gtk4::Button>(page_builder, "btn_gpu_screen_recorder");
    let window = window.clone();
    btn_gpu_screen_recorder.connect_clicked(move |_| {
        info!("Multimedia tools: GPU Screen Recorder button clicked");

        // Check official repos first; fall back to AUR if unavailable.
        let in_repos = core::is_package_in_repos("gpu-screen-recorder-gtk");

        let install_cmd = if in_repos {
            info!("gpu-screen-recorder-gtk found in official repos – installing via pacman");
            Command::builder()
                .privileged()
                .program("pacman")
                .args(&["-S", "--noconfirm", "--needed", "gpu-screen-recorder-gtk"])
                .description("Installing GPU Screen Recorder GTK from official repos...")
                .build()
        } else {
            info!("gpu-screen-recorder-gtk not in official repos – installing via AUR");
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "gpu-screen-recorder-gtk"])
                .description("Installing GPU Screen Recorder GTK from AUR...")
                .build()
        };

        let commands = CommandSequence::new().then(install_cmd).build();

        task_runner::run(window.upcast_ref(), commands, "GPU Screen Recorder Setup");
    });
}

fn setup_streaming_services(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_streaming = extract_widget::<gtk4::Button>(page_builder, "btn_streaming_services");
    let window = window.clone();

    btn_streaming.connect_clicked(move |_| {
        info!("Multimedia tools: Streaming Services button clicked");
        let window_ref = window.upcast_ref();

        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let is_steamos = std::path::Path::new("/usr/bin/steamos-add-to-steam").exists();
        if is_steamos {
            info!("Handheld device detected");
        }

        let apps_dir = if is_steamos {
            format!("{}/Applications", home)
        } else {
            format!("{}/.local/share/applications", home)
        };

        let dialog_desc = if is_steamos {
            "Select services to add as fullscreen Chrome kiosk web apps.\n\
             Flatpak Google Chrome will be installed if needed.\n\
             Handheld device detected — selected apps will be added to Steam."
        } else {
            "Select services to add as fullscreen Chrome kiosk web apps.\n\
             Flatpak Google Chrome will be installed if needed."
        };

        let mut config = SelectionDialogConfig::new(
            "Streaming Service Web Apps",
            dialog_desc,
        )
        .selection_type(SelectionType::Multi)
        .selection_required(true)
        .confirm_label("Add Selected");

        for (name, _url) in STREAMING_SERVICES {
            let desktop_path = format!("{}/{}.desktop", apps_dir, name);
            let installed = std::path::Path::new(&desktop_path).exists();
            config = config.add_option(SelectionOption::new(name, name, "", installed));
        }

        let window_for_closure = window.clone();
        show_selection_dialog(window_ref, config, move |selected_ids| {
            if selected_ids.is_empty() {
                return;
            }

            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            let is_steamos = std::path::Path::new("/usr/bin/steamos-add-to-steam").exists();

            let apps_dir = if is_steamos {
                format!("{}/Applications", home)
            } else {
                format!("{}/.local/share/applications", home)
            };

            let mut commands = CommandSequence::new();

            // Install Chrome flatpak if not present
            if !core::is_flatpak_installed("com.google.Chrome") {
                commands = commands.then(
                    Command::builder()
                        .normal()
                        .program("flatpak")
                        .args(&["install", "-y", "com.google.Chrome"])
                        .description("Installing Google Chrome (Flatpak)...")
                        .build(),
                );
            }

            // Flatpak overrides: udev for controller support (always)
            // + ~/Applications filesystem access on SteamOS
            if is_steamos {
                commands = commands.then(
                    Command::builder()
                        .normal()
                        .program("flatpak")
                        .args(&[
                            "override",
                            "--user",
                            "--filesystem=/run/udev:ro",
                            &format!("--filesystem={}/Applications", home),
                            "com.google.Chrome",
                        ])
                        .description("Handheld device detected, configuring Chrome permissions...")
                        .build(),
                );
            } else {
                commands = commands.then(
                    Command::builder()
                        .normal()
                        .program("flatpak")
                        .args(&[
                            "override",
                            "--user",
                            "--filesystem=/run/udev:ro",
                            "com.google.Chrome",
                        ])
                        .description("Configuring Chrome controller permissions...")
                        .build(),
                );
            }

            // Build a single shell script that creates all selected .desktop files
            let mut script_parts = vec![format!("mkdir -p '{}'", apps_dir)];

            for selected_name in &selected_ids {
                if let Some((name, url)) = STREAMING_SERVICES
                    .iter()
                    .find(|(n, _)| *n == selected_name.as_str())
                {
                    let desktop_path = format!("{}/{}.desktop", apps_dir, name);
                    script_parts.push(format!(
                        concat!(
                            "printf '%s\\n' ",
                            "'[Desktop Entry]' ",
                            "'Name={}' ",
                            "'Type=Application' ",
                            "'Icon=com.google.Chrome' ",
                            "'Exec=/usr/bin/flatpak run --branch=stable --arch=x86_64 ",
                            "com.google.Chrome --kiosk --start-fullscreen ",
                            "--force-device-scale-factor=1.5 \"{}\"' ",
                            "'Categories=Network;WebBrowser;' ",
                            "> '{}' && chmod 0644 '{}'"
                        ),
                        name, url, desktop_path, desktop_path
                    ));
                }
            }

            let full_script = script_parts.join(" && ");
            let desc = format!(
                "Creating {} streaming service web app(s)...",
                selected_ids.len()
            );

            commands = commands.then(
                Command::builder()
                    .normal()
                    .program("sh")
                    .args(&["-c", &full_script])
                    .description(&desc)
                    .build(),
            );

            // On SteamOS, add each .desktop file to Steam
            if is_steamos {
                let mut steam_parts = Vec::new();
                for selected_name in &selected_ids {
                    if let Some((name, _url)) = STREAMING_SERVICES
                        .iter()
                        .find(|(n, _)| *n == selected_name.as_str())
                    {
                        let desktop_path = format!("{}/{}.desktop", apps_dir, name);
                        steam_parts.push(format!(
                            "steamos-add-to-steam '{}' || true",
                            desktop_path
                        ));
                    }
                }

                if !steam_parts.is_empty() {
                    let steam_script = steam_parts.join(" && ");
                    commands = commands.then(
                        Command::builder()
                            .normal()
                            .program("sh")
                            .args(&["-c", &steam_script])
                            .description("Handheld device detected — adding web apps to Steam...")
                            .build(),
                    );
                }
            }

            task_runner::run(
                window_for_closure.upcast_ref(),
                commands.build(),
                "Streaming Services Setup",
            );
        });
    });
}
