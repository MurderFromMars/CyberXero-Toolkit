//! Drivers & hardware tools page.
//!
//! Five of the eight installers are a straight "click → run this AUR
//! sequence" — those are driven off a single [`SimpleSpec`] table to
//! eliminate copy-pasted boilerplate. The remaining three (OpenRazer,
//! NVIDIA CUDA, NVIDIA Legacy) open a dialog first, so they're wired up
//! explicitly.

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;

pub fn setup_handlers(
    page_builder: &Builder,
    _main_builder: &Builder,
    window: &ApplicationWindow,
) {
    wire_simple_handlers(page_builder, window);
    wire_openrazer(page_builder, window);
    wire_nvidia_legacy(page_builder, window);
    wire_cuda(page_builder, window);
}

// ---------------------------------------------------------------------------
// Simple single-click installers
// ---------------------------------------------------------------------------

/// Description of a "click a button, run a fixed sequence" installer.
/// `build` is a fn pointer so the `&'static` table below can hold it.
struct SimpleSpec {
    button_id: &'static str,
    title: &'static str,
    log: &'static str,
    build: fn() -> CommandSequence,
}

const SIMPLE: &[SimpleSpec] = &[
    SimpleSpec {
        button_id: "btn_tailscale",
        title: "Install Tailscale VPN",
        log: "Tailscale VPN",
        build: build_tailscale,
    },
    SimpleSpec {
        button_id: "btn_asus_rog",
        title: "Install ASUS ROG Tools",
        log: "ASUS ROG Tools",
        build: build_asus_rog,
    },
    SimpleSpec {
        button_id: "btn_cooler_control",
        title: "Install Cooler Control",
        log: "Cooler Control",
        build: build_cooler_control,
    },
    SimpleSpec {
        button_id: "btn_zenergy",
        title: "Install Zenergy Driver",
        log: "Zenergy Driver",
        build: build_zenergy,
    },
    SimpleSpec {
        button_id: "btn_rocm",
        title: "Install AMD ROCm",
        log: "AMD ROCm",
        build: build_rocm,
    },
];

fn wire_simple_handlers(builder: &Builder, window: &ApplicationWindow) {
    for spec in SIMPLE {
        let btn = extract_widget::<Button>(builder, spec.button_id);
        let window = window.clone();
        let title = spec.title;
        let log = spec.log;
        let build = spec.build;
        btn.connect_clicked(move |_| {
            info!("{log} button clicked");
            task_runner::run(window.upcast_ref(), build(), title);
        });
    }
}

fn build_tailscale() -> CommandSequence {
    CommandSequence::new()
        .then(priv_cmd(
            "bash",
            &[
                "-c",
                "curl -fsSL https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/install.sh | bash",
            ],
            "Installing Tailscale VPN...",
        ))
        .build()
}

fn build_asus_rog() -> CommandSequence {
    CommandSequence::new()
        .then(aur_install(
            &["rog-control-center", "asusctl", "supergfxctl"],
            "Installing ASUS ROG control tools...",
        ))
        .then(priv_cmd(
            "systemctl",
            &["enable", "--now", "asusd", "supergfxd"],
            "Enabling ASUS ROG services...",
        ))
        .build()
}

fn build_cooler_control() -> CommandSequence {
    CommandSequence::new()
        .then(aur_install(
            &["coolercontrol", "coolercontrold", "liquidctl"],
            "Installing Cooler Control daemon and liquidctl...",
        ))
        .then(priv_cmd(
            "systemctl",
            &["enable", "--now", "coolercontrold.service"],
            "Enabling Cooler Control daemon service...",
        ))
        .build()
}

fn build_zenergy() -> CommandSequence {
    CommandSequence::new()
        .then(aur_install(
            &["zenergy-dkms-git"],
            "Installing Zenergy Driver...",
        ))
        .build()
}

fn build_rocm() -> CommandSequence {
    CommandSequence::new()
        .then(aur_install(
            &["rocm-hip-sdk", "rocm-opencl-sdk"],
            "Installing AMD ROCm SDK...",
        ))
        .build()
}

// ---------------------------------------------------------------------------
// OpenRazer — pick optional frontend(s), add user to plugdev
// ---------------------------------------------------------------------------

fn wire_openrazer(builder: &Builder, window: &ApplicationWindow) {
    let btn = extract_widget::<Button>(builder, "btn_openrazer");
    let window = window.clone();
    btn.connect_clicked(move |_| {
        info!("OpenRazer button clicked");
        let window_inner = window.clone();
        let config = SelectionDialogConfig::new(
            "OpenRazer Drivers & Frontend",
            "OpenRazer drivers will be installed. Optionally select a frontend application for managing your Razer devices.",
        )
        .selection_type(SelectionType::Multi)
        .selection_required(false)
        .add_option(SelectionOption::new(
            "polychromatic",
            "Polychromatic",
            "Graphical frontend for managing Razer devices (GTK-based)",
            core::is_package_installed("polychromatic"),
        ))
        .add_option(SelectionOption::new(
            "razergenie",
            "RazerGenie",
            "Graphical frontend for managing Razer devices (Qt-based)",
            core::is_package_installed("razergenie"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |picked| {
            task_runner::run(
                window_inner.upcast_ref(),
                openrazer_plan(&picked),
                "Install OpenRazer Drivers (Reboot Required)",
            );
        });
    });
}

fn openrazer_plan(extras: &[String]) -> CommandSequence {
    let user = crate::config::env::get().user.clone();
    let mut seq = CommandSequence::new()
        .then(aur_install(
            &["openrazer-meta-git"],
            "Installing OpenRazer drivers...",
        ))
        .then(priv_cmd(
            "usermod",
            &["-aG", "plugdev", &user],
            "Adding user to plugdev group...",
        ));
    if extras.iter().any(|s| s == "polychromatic") {
        seq = seq.then(aur_install(
            &["polychromatic"],
            "Installing Polychromatic frontend...",
        ));
    }
    if extras.iter().any(|s| s == "razergenie") {
        seq = seq.then(aur_install(
            &["razergenie"],
            "Installing RazerGenie frontend...",
        ));
    }
    seq.build()
}

// ---------------------------------------------------------------------------
// NVIDIA Legacy (GTX 900/1000 series, 580xx branch)
// ---------------------------------------------------------------------------

const NVIDIA_LEGACY_WARNING: &str =
    "This is only intended for <span foreground=\"red\" weight=\"bold\">GTX900/1000</span> Series Legacy GPUs\n\
     For <span foreground=\"cyan\" weight=\"bold\">RTX/Turing+</span> GPUs download the <span foreground=\"green\" weight=\"bold\">nVidia</span> ISO instead.\n\n\
     <span foreground=\"red\" weight=\"bold\">No Support/Help</span> will be provided for those Legacy GPUs !";

const NVIDIA_LEGACY_PACKAGES: &[&str] = &[
    "lib32-nvidia-580xx-utils",
    "lib32-opencl-nvidia-580xx",
    "nvidia-580xx-dkms",
    "nvidia-580xx-utils",
    "opencl-nvidia-580xx",
];

const NVIDIA_LEGACY_SERVICES: &[&str] = &[
    "enable",
    "nvidia-suspend.service",
    "nvidia-hibernate.service",
    "nvidia-resume.service",
];

fn wire_nvidia_legacy(builder: &Builder, window: &ApplicationWindow) {
    let btn = extract_widget::<Button>(builder, "btn_nvidia_legacy");
    let window = window.clone();
    btn.connect_clicked(move |_| {
        info!("Nvidia Legacy button clicked");
        let window_inner = window.clone();
        show_warning_confirmation(
            window.upcast_ref(),
            "Nvidia Legacy Drivers",
            NVIDIA_LEGACY_WARNING,
            move || {
                task_runner::run(
                    window_inner.upcast_ref(),
                    nvidia_legacy_plan(),
                    "Install Nvidia Legacy Drivers",
                );
            },
        );
    });
}

fn nvidia_legacy_plan() -> CommandSequence {
    let scripts = crate::config::paths::scripts();
    let grub = scripts
        .join("nvidia_grub.sh")
        .to_string_lossy()
        .into_owned();
    let mkinitcpio = scripts
        .join("nvidia_mkinitcpio.sh")
        .to_string_lossy()
        .into_owned();

    CommandSequence::new()
        .then(aur_install(
            NVIDIA_LEGACY_PACKAGES,
            "Installing Nvidia Legacy Drivers...",
        ))
        .then(priv_cmd(
            "bash",
            &[&grub],
            "Configuring GRUB (nvidia-drm.modeset=1)...",
        ))
        .then(priv_cmd(
            "bash",
            &[&mkinitcpio],
            "Configuring mkinitcpio modules...",
        ))
        .then(priv_cmd(
            "systemctl",
            NVIDIA_LEGACY_SERVICES,
            "Enabling Nvidia power management services...",
        ))
        .then(priv_cmd(
            "mkinitcpio",
            &["-P"],
            "Rebuilding initramfs...",
        ))
        .build()
}

// ---------------------------------------------------------------------------
// NVIDIA CUDA — user picks a version
// ---------------------------------------------------------------------------

fn wire_cuda(builder: &Builder, window: &ApplicationWindow) {
    let btn = extract_widget::<Button>(builder, "btn_cuda");
    let window = window.clone();
    btn.connect_clicked(move |_| {
        info!("NVIDIA CUDA button clicked");
        let window_inner = window.clone();
        let config = SelectionDialogConfig::new(
            "NVIDIA CUDA Toolkit",
            "Select the CUDA version to install. The latest version is recommended for most users.",
        )
        .selection_type(SelectionType::Single)
        .selection_required(true)
        .add_option(SelectionOption::new(
            "cuda",
            "CUDA (Latest)",
            "Install the latest CUDA toolkit from official repositories",
            core::is_package_installed("cuda"),
        ))
        .add_option(SelectionOption::new(
            "cuda-12.9",
            "CUDA 12.9",
            "Install CUDA Toolkit version 12.9 specifically",
            core::is_package_installed("cuda-12.9"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |picked| {
            let Some(package) = picked.first() else { return };
            let description = format!("Installing {package}...");
            let seq = CommandSequence::new()
                .then(aur_install(&[package.as_str()], &description))
                .build();
            task_runner::run(window_inner.upcast_ref(), seq, "Install NVIDIA CUDA");
        });
    });
}

// ---------------------------------------------------------------------------
// Command construction helpers
// ---------------------------------------------------------------------------

/// Build an `aur -S --noconfirm --needed <packages>` command.
fn aur_install(packages: &[&str], description: &str) -> Command {
    let mut args: Vec<&str> = vec!["-S", "--noconfirm", "--needed"];
    args.extend_from_slice(packages);
    Command::builder()
        .aur()
        .args(&args)
        .description(description)
        .build()
}

/// Build a privileged command routed through the auth daemon.
fn priv_cmd(program: &str, args: &[&str], description: &str) -> Command {
    Command::builder()
        .privileged()
        .program(program)
        .args(args)
        .description(description)
        .build()
}
