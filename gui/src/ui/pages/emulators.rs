//! Emulators page button handlers.
//!
//! Handles:
//! - RetroArch with selectable libretro cores
//! - Standalone emulator installation

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

// ── Public setup ─────────────────────────────────────────────────────────────

/// Set up all button handlers for the emulators page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_retroarch(page_builder, window);
    setup_standalone(page_builder, window, "btn_emu_ps1", StandaloneEmu::DuckStation);
    setup_standalone(page_builder, window, "btn_emu_ps2", StandaloneEmu::Pcsx2);
    setup_standalone(page_builder, window, "btn_emu_ps3", StandaloneEmu::Rpcs3);
    setup_standalone(page_builder, window, "btn_emu_ps4", StandaloneEmu::ShadPs4);
    setup_standalone(page_builder, window, "btn_emu_psp", StandaloneEmu::Ppsspp);
    setup_standalone(page_builder, window, "btn_emu_vita", StandaloneEmu::Vita3k);
    setup_standalone(page_builder, window, "btn_emu_gba", StandaloneEmu::Mgba);
    setup_standalone(page_builder, window, "btn_emu_nds", StandaloneEmu::Melonds);
    setup_standalone(page_builder, window, "btn_emu_dolphin", StandaloneEmu::Dolphin);
    setup_standalone(page_builder, window, "btn_emu_wiiu", StandaloneEmu::Cemu);
    setup_standalone(page_builder, window, "btn_emu_switch", StandaloneEmu::Ryujinx);
    setup_standalone(page_builder, window, "btn_emu_xbox", StandaloneEmu::Xemu);
    setup_standalone(page_builder, window, "btn_emu_dreamcast", StandaloneEmu::Flycast);
    setup_standalone(page_builder, window, "btn_emu_mame", StandaloneEmu::Mame);
}

// ── RetroArch + Cores ────────────────────────────────────────────────────────

fn setup_retroarch(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_emu_retroarch");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Emulators: RetroArch button clicked");
        let window_ref = window.upcast_ref();

        let config = SelectionDialogConfig::new(
            "RetroArch — Select Cores",
            "RetroArch will be installed with assets.\n\
             Select additional libretro cores below.",
        )
        .selection_type(SelectionType::Multi)
        .selection_required(false)
        .add_option(SelectionOption::new(
            "nes", "NES (Mesen)",
            "libretro-mesen — accurate NES/Famicom emulation",
            core::is_package_installed("libretro-mesen"),
        ))
        .add_option(SelectionOption::new(
            "snes", "SNES (bsnes-hd)",
            "libretro-bsnes-hd — SNES with HD mode 7",
            core::is_package_installed("libretro-bsnes-hd"),
        ))
        .add_option(SelectionOption::new(
            "snes9x", "SNES (Snes9x)",
            "libretro-snes9x — fast SNES emulation",
            core::is_package_installed("libretro-snes9x"),
        ))
        .add_option(SelectionOption::new(
            "gb", "Game Boy / Color (Gambatte)",
            "libretro-gambatte — accurate GB/GBC",
            core::is_package_installed("libretro-gambatte"),
        ))
        .add_option(SelectionOption::new(
            "gba", "GBA (mGBA)",
            "libretro-mgba — Game Boy Advance",
            core::is_package_installed("libretro-mgba"),
        ))
        .add_option(SelectionOption::new(
            "n64", "N64 (Mupen64Plus-Next)",
            "libretro-mupen64plus-next — Nintendo 64",
            core::is_package_installed("libretro-mupen64plus-next"),
        ))
        .add_option(SelectionOption::new(
            "nds", "DS (melonDS)",
            "libretro-melonds — Nintendo DS",
            core::is_package_installed("libretro-melonds"),
        ))
        .add_option(SelectionOption::new(
            "psx", "PS1 (Beetle PSX HW)",
            "libretro-beetle-psx-hw — hardware-accelerated PS1",
            core::is_package_installed("libretro-beetle-psx-hw"),
        ))
        .add_option(SelectionOption::new(
            "genesis", "Genesis / Mega Drive",
            "libretro-genesis-plus-gx — Sega Genesis, Game Gear, Master System",
            core::is_package_installed("libretro-genesis-plus-gx"),
        ))
        .add_option(SelectionOption::new(
            "picodrive", "Genesis / 32X (PicoDrive)",
            "libretro-picodrive — Sega 32X support",
            core::is_package_installed("libretro-picodrive"),
        ))
        .add_option(SelectionOption::new(
            "saturn", "Saturn (Kronos)",
            "libretro-kronos — Sega Saturn / ST-V",
            core::is_package_installed("libretro-kronos"),
        ))
        .add_option(SelectionOption::new(
            "dc", "Dreamcast (Flycast)",
            "libretro-flycast — Sega Dreamcast / Naomi",
            core::is_package_installed("libretro-flycast"),
        ))
        .add_option(SelectionOption::new(
            "pce", "PC Engine / TurboGrafx-16",
            "libretro-beetle-pce — NEC PC Engine",
            core::is_package_installed("libretro-beetle-pce"),
        ))
        .add_option(SelectionOption::new(
            "ppsspp", "PSP (PPSSPP)",
            "libretro-ppsspp — PlayStation Portable",
            core::is_package_installed("libretro-ppsspp"),
        ))
        .add_option(SelectionOption::new(
            "desmume", "DS (DeSmuME)",
            "libretro-desmume — alternate Nintendo DS core",
            core::is_package_installed("libretro-desmume"),
        ))
        .add_option(SelectionOption::new(
            "mame", "Arcade (MAME)",
            "libretro-mame — arcade machines",
            core::is_package_installed("libretro-mame"),
        ))
        .add_option(SelectionOption::new(
            "scummvm", "ScummVM",
            "libretro-scummvm — classic point-and-click adventures",
            core::is_package_installed("libretro-scummvm"),
        ))
        .add_option(SelectionOption::new(
            "atari2600", "Atari 2600 (Stella)",
            "libretro-stella — Atari 2600",
            core::is_package_installed("libretro-stella"),
        ))
        .add_option(SelectionOption::new(
            "atari7800", "Atari 7800 (ProSystem)",
            "libretro-prosystem — Atari 7800",
            core::is_package_installed("libretro-prosystem"),
        ))
        .add_option(SelectionOption::new(
            "atarilynx", "Atari Lynx (Handy)",
            "libretro-handy — Atari Lynx",
            core::is_package_installed("libretro-handy"),
        ))
        .add_option(SelectionOption::new(
            "atarijaguar", "Atari Jaguar (Virtual Jaguar)",
            "libretro-virtualjaguar — Atari Jaguar",
            core::is_package_installed("libretro-virtualjaguar"),
        ))
        .add_option(SelectionOption::new(
            "atari5200", "Atari 5200/800 (Atari800)",
            "libretro-atari800 — Atari 5200 / 800 / XL / XE",
            core::is_package_installed("libretro-atari800"),
        ))
        .add_option(SelectionOption::new(
            "3do", "3DO (Opera)",
            "libretro-opera — 3DO Interactive Multiplayer",
            core::is_package_installed("libretro-opera"),
        ))
        .add_option(SelectionOption::new(
            "amiga", "Amiga (PUAE)",
            "libretro-puae — Commodore Amiga",
            core::is_package_installed("libretro-puae"),
        ))
        .add_option(SelectionOption::new(
            "c64", "C64 (VICE)",
            "libretro-vice-x64 — Commodore 64",
            core::is_package_installed("libretro-vice-x64"),
        ))
        .add_option(SelectionOption::new(
            "dos", "DOS (DOSBox Pure)",
            "libretro-dosbox-pure — MS-DOS games",
            core::is_package_installed("libretro-dosbox-pure"),
        ))
        .add_option(SelectionOption::new(
            "ngp", "Neo Geo Pocket (Beetle NGP)",
            "libretro-beetle-ngp — Neo Geo Pocket / Color",
            core::is_package_installed("libretro-beetle-ngp"),
        ))
        .add_option(SelectionOption::new(
            "wonderswan", "WonderSwan (Beetle WS)",
            "libretro-beetle-wswan — Bandai WonderSwan / Color",
            core::is_package_installed("libretro-beetle-wswan"),
        ))
        .add_option(SelectionOption::new(
            "shaders", "Slang Shaders",
            "libretro-shaders-slang — CRT filters, scanlines, etc.",
            core::is_package_installed("libretro-shaders-slang"),
        ))
        .confirm_label("Install");

        let window_for_closure = window.clone();
        show_selection_dialog(window_ref, config, move |selected_ids| {
            let mut commands = CommandSequence::new();

            // Always install RetroArch base + assets
            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&[
                        "-S", "--noconfirm", "--needed",
                        "retroarch",
                        "retroarch-assets-ozone",
                        "retroarch-assets-xmb",
                        "libretro-core-info",
                    ])
                    .description("Installing RetroArch and assets...")
                    .build(),
            );

            // Map selection IDs to package names
            let core_map: &[(&str, &[&str])] = &[
                ("nes", &["libretro-mesen"]),
                ("snes", &["libretro-bsnes-hd"]),
                ("snes9x", &["libretro-snes9x"]),
                ("gb", &["libretro-gambatte"]),
                ("gba", &["libretro-mgba"]),
                ("n64", &["libretro-mupen64plus-next"]),
                ("nds", &["libretro-melonds"]),
                ("psx", &["libretro-beetle-psx-hw"]),
                ("genesis", &["libretro-genesis-plus-gx"]),
                ("picodrive", &["libretro-picodrive"]),
                ("saturn", &["libretro-kronos"]),
                ("dc", &["libretro-flycast"]),
                ("pce", &["libretro-beetle-pce"]),
                ("ppsspp", &["libretro-ppsspp"]),
                ("desmume", &["libretro-desmume"]),
                ("mame", &["libretro-mame"]),
                ("scummvm", &["libretro-scummvm"]),
                ("atari2600", &["libretro-stella"]),
                ("atari7800", &["libretro-prosystem"]),
                ("atarilynx", &["libretro-handy"]),
                ("atarijaguar", &["libretro-virtualjaguar"]),
                ("atari5200", &["libretro-atari800"]),
                ("3do", &["libretro-opera"]),
                ("amiga", &["libretro-puae"]),
                ("c64", &["libretro-vice-x64"]),
                ("dos", &["libretro-dosbox-pure"]),
                ("ngp", &["libretro-beetle-ngp"]),
                ("wonderswan", &["libretro-beetle-wswan"]),
                ("shaders", &["libretro-shaders-slang"]),
            ];

            let mut core_packages: Vec<&str> = Vec::new();
            for (id, pkgs) in core_map {
                if selected_ids.iter().any(|s| s == *id) {
                    core_packages.extend_from_slice(pkgs);
                }
            }

            if !core_packages.is_empty() {
                let mut args = vec!["-S", "--noconfirm", "--needed"];
                args.extend(core_packages.iter());

                commands = commands.then(
                    Command::builder()
                        .privileged()
                        .program("pacman")
                        .args(&args)
                        .description("Installing selected libretro cores...")
                        .build(),
                );
            }

            task_runner::run(
                window_for_closure.upcast_ref(),
                commands.build(),
                "RetroArch Installation",
            );
        });
    });
}

// ── Standalone Emulators ─────────────────────────────────────────────────────

enum StandaloneEmu {
    DuckStation,
    Pcsx2,
    Rpcs3,
    ShadPs4,
    Ppsspp,
    Vita3k,
    Mgba,
    Melonds,
    Dolphin,
    Cemu,
    Ryujinx,
    Xemu,
    Flycast,
    Mame,
}

impl StandaloneEmu {
    fn label(&self) -> &'static str {
        match self {
            Self::DuckStation => "DuckStation (PS1)",
            Self::Pcsx2 => "PCSX2 (PS2)",
            Self::Rpcs3 => "RPCS3 (PS3)",
            Self::ShadPs4 => "ShadPS4 (PS4)",
            Self::Ppsspp => "PPSSPP (PSP)",
            Self::Vita3k => "Vita3K (PS Vita)",
            Self::Mgba => "mGBA (GBA)",
            Self::Melonds => "melonDS (DS)",
            Self::Dolphin => "Dolphin (GC/Wii)",
            Self::Cemu => "Cemu (Wii U)",
            Self::Ryujinx => "Ryujinx (Switch)",
            Self::Xemu => "xemu (Xbox)",
            Self::Flycast => "Flycast (Dreamcast)",
            Self::Mame => "MAME (Arcade)",
        }
    }

    fn packages(&self) -> (Vec<&'static str>, Vec<&'static str>) {
        match self {
            Self::DuckStation => (vec![], vec!["duckstation-gpl"]),
            Self::Pcsx2 => (vec![], vec!["pcsx2-git"]),
            Self::Rpcs3 => (vec![], vec!["rpcs3-git"]),
            Self::ShadPs4 => (vec![], vec!["shadps4"]),
            Self::Ppsspp => (vec!["ppsspp"], vec![]),
            Self::Vita3k => (vec![], vec!["vita3k-git"]),
            Self::Mgba => (vec!["mgba-qt"], vec![]),
            Self::Melonds => (vec![], vec!["melonds-git"]),
            Self::Dolphin => (vec!["dolphin-emu"], vec![]),
            Self::Cemu => (vec![], vec!["cemu-git"]),
            Self::Ryujinx => (vec![], vec!["ryujinx"]),
            Self::Xemu => (vec![], vec!["xemu-git"]),
            Self::Flycast => (vec![], vec!["flycast"]),
            Self::Mame => (vec!["mame"], vec![]),
        }
    }
}

fn setup_standalone(
    builder: &Builder,
    window: &ApplicationWindow,
    button_id: &str,
    emu: StandaloneEmu,
) {
    let button = extract_widget::<Button>(builder, button_id);
    let window = window.clone();
    let label = emu.label();
    let (repo_pkgs, aur_pkgs) = emu.packages();

    button.connect_clicked(move |_| {
        info!("Emulators: {} button clicked", label);

        let mut commands = CommandSequence::new();

        if !repo_pkgs.is_empty() {
            let mut args = vec!["-S", "--noconfirm", "--needed"];
            args.extend(repo_pkgs.iter());

            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&args)
                    .description(&format!("Installing {} from repos...", label))
                    .build(),
            );
        }

        if !aur_pkgs.is_empty() {
            let mut args = vec!["-S", "--noconfirm", "--needed"];
            args.extend(aur_pkgs.iter());

            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&args)
                    .description(&format!("Installing {} from AUR...", label))
                    .build(),
            );
        }

        task_runner::run(
            window.upcast_ref(),
            commands.build(),
            &format!("{} Installation", label),
        );
    });
}
