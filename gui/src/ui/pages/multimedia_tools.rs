//! Multimedia tools page button handlers.
//!
//! Handles:
//! - OBS-Studio with plugins and V4L2
//! - Kdenlive video editor
//! - Jellyfin server installation
//! - GPU Screen Recorder GTK (repo-first, AUR fallback)
//! - Streaming service web app installer
//! - Enhanced Audio (PipeWire spatial convolver)

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
    setup_enhanced_audio(page_builder, window);
}

fn setup_obs_studio_aio(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_obs_studio_aio = extract_widget::<gtk4::Button>(page_builder, "btn_obs_studio_aio");
    let window = window.clone();
    btn_obs_studio_aio.connect_clicked(move |_| {
        info!("Multimedia tools: OBS-Studio AiO button clicked");
        let window_ref = window.upcast_ref();

        // State detection via pacman/-Q — all packages are native Arch/AUR.
        // obs-websocket has been bundled with obs-studio since v28, so it is
        // intentionally omitted as a standalone option.
        let obs_installed = core::is_package_installed("obs-studio");

        let graphics_capture_installed =
            core::is_package_installed("obs-vkcapture") &&
            core::is_package_installed("lib32-obs-vkcapture") &&
            core::is_package_installed("obs-gstreamer") &&
            core::is_package_installed("obs-vaapi");

        let transitions_effects_installed =
            core::is_package_installed("obs-move-transition") &&
            core::is_package_installed("obs-transition-table") &&
            core::is_package_installed("obs-scale-to-sound");

        let streaming_tools_installed =
            core::is_package_installed("obs-advanced-scene-switcher") &&
            core::is_package_installed("droidcam-obs");

        let audio_video_tools_installed =
            core::is_package_installed("obs-waveform") &&
            core::is_package_installed("obs-vertical-canvas") &&
            core::is_package_installed("obs-backgroundremoval");

        let v4l2_installed = core::is_package_installed("v4l2loopback-dkms");

        let config = SelectionDialogConfig::new(
            "OBS-Studio & Plugins Installation",
            "OBS-Studio will be installed from repos. Optionally select plugins to install.",
        )
        .selection_type(SelectionType::Multi)
        .selection_required(false)
        .add_option(SelectionOption::new(
            "graphics_capture",
            "Graphics Capture Plugins",
            "obs-vkcapture (32 & 64-bit), obs-gstreamer, obs-vaapi",
            graphics_capture_installed,
        ))
        .add_option(SelectionOption::new(
            "transitions_effects",
            "Transitions & Effects",
            "obs-move-transition, obs-transition-table, obs-scale-to-sound",
            transitions_effects_installed,
        ))
        .add_option(SelectionOption::new(
            "streaming_tools",
            "Streaming & Recording Tools",
            "obs-advanced-scene-switcher, droidcam-obs",
            streaming_tools_installed,
        ))
        .add_option(SelectionOption::new(
            "audio_video_tools",
            "Audio & Video Tools",
            "obs-waveform, obs-vertical-canvas, obs-backgroundremoval",
            audio_video_tools_installed,
        ))
        .add_option(SelectionOption::new(
            "v4l2",
            "V4L2loopback Virtual Camera",
            "Enable OBS virtual camera functionality",
            v4l2_installed,
        ))
        .confirm_label(if obs_installed { "Update" } else { "Install" });

        let window_for_closure = window.clone();
        show_selection_dialog(window_ref, config, move |selected_ids| {
            let mut commands = CommandSequence::new();

            // Always install / refresh obs-studio from repos
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "obs-studio"])
                    .description("Installing OBS-Studio...")
                    .build(),
            );

            if selected_ids.iter().any(|s| s == "graphics_capture") {
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&[
                            "-S", "--noconfirm", "--needed",
                            "obs-vkcapture",
                            "lib32-obs-vkcapture",
                            "obs-gstreamer",
                            "obs-vaapi",
                        ])
                        .description("Installing graphics capture plugins...")
                        .build(),
                );
            }

            if selected_ids.iter().any(|s| s == "transitions_effects") {
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&[
                            "-S", "--noconfirm", "--needed",
                            "obs-move-transition",
                            "obs-transition-table",
                            "obs-scale-to-sound",
                        ])
                        .description("Installing transitions & effects plugins...")
                        .build(),
                );
            }

            if selected_ids.iter().any(|s| s == "streaming_tools") {
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&[
                            "-S", "--noconfirm", "--needed",
                            "obs-advanced-scene-switcher",
                            "droidcam-obs",
                        ])
                        .description("Installing streaming & recording tools...")
                        .build(),
                );
            }

            if selected_ids.iter().any(|s| s == "audio_video_tools") {
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&[
                            "-S", "--noconfirm", "--needed",
                            "obs-waveform",
                            "obs-vertical-canvas",
                            "obs-backgroundremoval",
                        ])
                        .description("Installing audio/video enhancement plugins...")
                        .build(),
                );
            }

            if selected_ids.iter().any(|s| s == "v4l2") {
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "v4l2loopback-dkms", "v4l2loopback-utils"])
                        .description("Installing V4L2 loopback modules...")
                        .build(),
                );
                commands = commands.then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&["-c", "echo 'v4l2loopback' > /etc/modules-load.d/v4l2loopback.conf"])
                        .description("Enabling V4L2 loopback module at boot...")
                        .build(),
                );
                commands = commands.then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "echo 'options v4l2loopback exclusive_caps=1 card_label=\"OBS Virtual Camera\"' > /etc/modprobe.d/v4l2loopback.conf",
                        ])
                        .description("Configuring virtual camera options...")
                        .build(),
                );
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

// ── Enhanced Audio ────────────────────────────────────────────────────────────

const ENHANCED_AUDIO_CONF: &str =
    ".config/pipewire/pipewire.conf.d/spatial-audio.conf";
const SUSPEND_FIX_SERVICE: &str =
    "/etc/systemd/system/pipewire-fix-audio-after-suspend.service";
const ENHANCED_AUDIO_REPO: &str =
    "https://github.com/MurderFromMars/Enhanced-Handheld-Audio/archive/main.tar.gz";

/// Returns the active intensity ("light" / "medium" / "heavy") if Enhanced
/// Audio is installed, or `None` if it isn't.
fn detect_enhanced_audio_intensity() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let conf_path = format!("{}/{}", home, ENHANCED_AUDIO_CONF);
    if !std::path::Path::new(&conf_path).exists() {
        return None;
    }
    if let Ok(content) = std::fs::read_to_string(&conf_path) {
        for line in content.lines() {
            if line.contains("# Intensity:") {
                for level in &["light", "medium", "heavy"] {
                    if line.contains(level) {
                        return Some(level.to_string());
                    }
                }
                // Conf exists but intensity line unrecognised — still installed.
                return Some("unknown".to_string());
            }
        }
        // Conf exists but no intensity comment — treat as installed.
        return Some("unknown".to_string());
    }
    None
}

/// Returns `true` if the suspend-fix systemd service is present.
fn detect_suspend_fix() -> bool {
    std::path::Path::new(SUSPEND_FIX_SERVICE).exists()
}

/// Returns a list of available ALSA output sink node names via pactl.
/// Each entry is (node_name, friendly_description).
fn detect_audio_sinks() -> Vec<(String, String)> {
    let output = std::process::Command::new("pactl")
        .args(["list", "sinks"])
        .output();

    let Ok(output) = output else { return Vec::new() };
    let text = String::from_utf8_lossy(&output.stdout);

    let mut sinks: Vec<(String, String)> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_desc: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("Name: ") {
            // Flush previous sink if complete
            if let (Some(n), Some(d)) = (current_name.take(), current_desc.take()) {
                if n.starts_with("alsa_output") {
                    sinks.push((n, d));
                }
            }
            current_name = Some(name.to_string());
            current_desc = None;
        } else if let Some(desc) = trimmed.strip_prefix("Description: ") {
            current_desc = Some(desc.to_string());
        }
    }
    // Flush last sink
    if let (Some(n), Some(d)) = (current_name, current_desc) {
        if n.starts_with("alsa_output") {
            sinks.push((n, d));
        }
    }
    sinks
}

/// Returns a label string for an intensity option, appending "(active)" when
/// it matches the currently installed intensity.
fn intensity_label(level: &str, current: Option<&str>) -> String {
    if current == Some(level) {
        format!("{} (active)", level_display(level))
    } else {
        level_display(level).to_string()
    }
}

fn level_display(level: &str) -> &str {
    match level {
        "light"  => "Light",
        "medium" => "Medium",
        "heavy"  => "Heavy",
        _        => level,
    }
}

fn setup_enhanced_audio(page_builder: &Builder, window: &ApplicationWindow) {
    let btn = extract_widget::<gtk4::Button>(page_builder, "btn_enhanced_audio");
    let window = window.clone();

    btn.connect_clicked(move |_| {
        info!("Multimedia tools: Enhanced Audio button clicked");
        let window_ref = window.upcast_ref();

        let current_intensity = detect_enhanced_audio_intensity();
        let is_installed      = current_intensity.is_some();
        let suspend_installed = detect_suspend_fix();
        let current_str       = current_intensity.as_deref();

        // ── Dialog 1: intensity selection (radio, required) ──────────────────
        //
        // All options have `installed: false` so nothing is pre-locked; the
        // active level is indicated in the label text instead.
        // If Enhanced Audio is already installed an "Uninstall" radio is added
        // at the bottom — choosing it skips dialog 2 entirely.

        let mut intensity_config = SelectionDialogConfig::new(
            "Enhanced Audio",
            "Select spatial intensity for your built-in speakers.\n\
             Installs a virtual PipeWire sink via 4-channel crossfeed convolution.",
        )
        .selection_type(SelectionType::Single)
        .selection_required(true)
        .add_option(SelectionOption::new(
            "light",
            &intensity_label("light", current_str),
            "15% crossfeed · 3 reflections · best for music",
            false,
        ))
        .add_option(SelectionOption::new(
            "medium",
            &intensity_label("medium", current_str),
            "25% crossfeed · 5 reflections · general gaming and media",
            false,
        ))
        .add_option(SelectionOption::new(
            "heavy",
            &intensity_label("heavy", current_str),
            "35% crossfeed · 7 reflections · single-player / movies",
            false,
        ));

        if is_installed {
            intensity_config = intensity_config.add_option(SelectionOption::new(
                "uninstall",
                "Uninstall Enhanced Audio",
                "Remove all config files and restore default audio",
                false,
            ));
        }

        intensity_config =
            intensity_config.confirm_label(if is_installed { "Next →" } else { "Next →" });

        let window_for_extras = window.clone();

        show_selection_dialog(window_ref, intensity_config, move |intensity_selected| {
            let choice = intensity_selected
                .into_iter()
                .next()
                .unwrap_or_else(|| "medium".to_string());

            // ── Uninstall path — run immediately, no further dialogs ──────────
            if choice == "uninstall" {
                let script = format!(
                    "export TERM=xterm-256color; \
                     tmp=$(mktemp -d) && \
                     curl -fsSL '{repo}' | tar -xz -C \"$tmp\" --strip-components=1 && \
                     \"$tmp/install.sh\" --uninstall && \
                     rm -rf \"$tmp\"",
                    repo = ENHANCED_AUDIO_REPO,
                );
                let commands = CommandSequence::new()
                    .then(
                        Command::builder()
                            .normal()
                            .program("sh")
                            .args(&["-c", &script])
                            .description("Removing Enhanced Audio...")
                            .build(),
                    )
                    .build();

                task_runner::run(
                    window_for_extras.upcast_ref(),
                    commands,
                    "Enhanced Audio — Uninstall",
                );
                return;
            }

            // ── Dialog 2: sink selection (only when >1 sink exists) ───────────
            let intensity = choice;
            let sinks = detect_audio_sinks();

            if sinks.len() > 1 {
                // Multiple sinks — let the user pick one.
                let mut sink_config = SelectionDialogConfig::new(
                    "Enhanced Audio — Select Output Device",
                    "Multiple audio outputs detected. Choose which device to enhance.\n\
                     Recommend built-in speakers or analog-stereo output.",
                )
                .selection_type(SelectionType::Single)
                .selection_required(true)
                .confirm_label("Next →");

                for (node_name, description) in &sinks {
                    sink_config = sink_config.add_option(SelectionOption::new(
                        node_name,
                        description,
                        node_name,
                        false,
                    ));
                }

                let window_for_extras2 = window_for_extras.clone();
                let intensity_for_sink = intensity.clone();

                show_selection_dialog(
                    window_for_extras.upcast_ref(),
                    sink_config,
                    move |sink_selected| {
                        let sink = sink_selected.into_iter().next().unwrap_or_default();
                        show_enhanced_audio_extras_dialog(
                            &window_for_extras2,
                            intensity_for_sink.clone(),
                            sink,
                            suspend_installed,
                            is_installed,
                        );
                    },
                );
            } else {
                // Zero or one sink — pass it directly (empty string = auto-detect).
                let sink = sinks.into_iter().next().map(|(n, _)| n).unwrap_or_default();
                show_enhanced_audio_extras_dialog(
                    &window_for_extras,
                    intensity,
                    sink,
                    suspend_installed,
                    is_installed,
                );
            }
        });
    });
}

/// Shows the extras dialog and then runs the install command.
fn show_enhanced_audio_extras_dialog(
    window: &gtk4::ApplicationWindow,
    intensity: String,
    sink: String,
    suspend_installed: bool,
    is_installed: bool,
) {
    let extras_config = SelectionDialogConfig::new(
        "Enhanced Audio — Options",
        "Optional extras. Leave everything unchecked to skip.",
    )
    .selection_type(SelectionType::Multi)
    .selection_required(false)
    .add_option(SelectionOption::new(
        "suspend_fix",
        "Suspend / Resume Audio Fix",
        "Systemd service that fixes crackling or fuzzy audio after sleep",
        suspend_installed,
    ))
    .confirm_label(if is_installed { "Update" } else { "Install" });

    let window_for_run = window.clone();

    show_selection_dialog(window.upcast_ref(), extras_config, move |extras_selected| {
        let suspend_flag = if extras_selected.iter().any(|s| s == "suspend_fix") {
            " --suspend-fix"
        } else {
            ""
        };

        let sink_flag = if sink.is_empty() {
            String::new()
        } else {
            format!(" --sink '{}'", sink)
        };

        let script = format!(
            "export TERM=xterm-256color; \
             tmp=$(mktemp -d) && \
             curl -fsSL '{repo}' | tar -xz -C \"$tmp\" --strip-components=1 && \
             chmod +x \"$tmp/install.sh\" && \
             \"$tmp/install.sh\" --intensity {intensity}{sink}{suspend} && \
             rm -rf \"$tmp\"",
            repo      = ENHANCED_AUDIO_REPO,
            intensity = intensity,
            sink      = sink_flag,
            suspend   = suspend_flag,
        );

        let desc = format!(
            "{} Enhanced Audio ({} intensity)...",
            if is_installed { "Updating" } else { "Installing" },
            intensity,
        );

        let mut commands = CommandSequence::new();

        // The install script calls sudo internally for the suspend fix systemd
        // service, which is intercepted by the toolkit's sudo shim and routed
        // through xero-auth. Start the daemon with a no-op privileged command
        // first so the shim finds it running when the script invokes sudo.
        if suspend_flag.contains("suspend-fix") {
            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("true")
                    .args(&[])
                    .description("Requesting elevated privileges for suspend fix...")
                    .build(),
            );
        }

        commands = commands.then(
            Command::builder()
                .normal()
                .program("sh")
                .args(&["-c", &script])
                .description(&desc)
                .build(),
        );

        task_runner::run(
            window_for_run.upcast_ref(),
            commands.build(),
            "Enhanced Audio Setup",
        );
    });
}
