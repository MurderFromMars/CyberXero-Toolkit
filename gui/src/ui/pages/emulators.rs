//! Emulators page button handlers.
//!
//! Handles:
//! - Unified emulation filesystem setup (EmuDeck-style ~/Emulation/ tree)
//! - RetroArch with selectable libretro cores + config patching
//! - Standalone emulators with post-install config pointing at ~/Emulation/

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// ROM subdirectories created under ~/Emulation/roms/.
const ROM_DIRS: &[&str] = &[
    "3do", "amiga", "arcade", "atari2600", "atari5200", "atari7800",
    "atarijaguar", "atarilynx", "c64", "dc", "dos", "gb", "gba", "gbc", "gc",
    "genesis", "mastersystem", "n64", "nds", "nes", "ngp", "pce", "ps1", "ps2",
    "ps3", "ps4", "psp", "psvita", "saturn", "scummvm", "segacd", "snes",
    "switch", "wii", "wiiu", "wonderswan", "xbox",
];

/// Per-emulator save subdirectories under ~/Emulation/saves/.
const SAVE_DIRS: &[&str] = &[
    "retroarch", "duckstation", "pcsx2", "rpcs3", "shadps4", "ppsspp",
    "vita3k", "dolphin", "cemu", "ryujinx", "xemu", "flycast", "mame",
    "mgba", "melonds",
];

/// Per-emulator state subdirectories under ~/Emulation/states/.
const STATE_DIRS: &[&str] = &[
    "retroarch", "duckstation", "pcsx2", "ppsspp", "dolphin",
    "mgba", "melonds",
];

/// Build the shell command that creates the full ~/Emulation/ tree.
fn build_mkdir_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let mut dirs = vec![
        format!("{}/bios", base),
        format!("{}/screenshots", base),
        format!("{}/storage", base),
        format!("{}/tools", base),
    ];
    for d in ROM_DIRS {
        dirs.push(format!("{}/roms/{}", base, d));
    }
    for d in SAVE_DIRS {
        dirs.push(format!("{}/saves/{}", base, d));
    }
    for d in STATE_DIRS {
        dirs.push(format!("{}/states/{}", base, d));
    }
    format!("mkdir -p {}", dirs.join(" "))
}

// ── Config patching scripts ──────────────────────────────────────────────────
//
// Each function returns a shell script that creates/patches the emulator's
// config to use ~/Emulation/ paths. The scripts are idempotent — safe to
// re-run. They use a sed-or-append pattern: if the key exists, replace it;
// otherwise append it.

/// Helper: generates a sed command that sets key=value in an INI-style file,
/// appending if the key doesn't exist yet.
fn ini_set(file: &str, key: &str, value: &str) -> String {
    // If the key exists (possibly commented), replace the line; otherwise append.
    format!(
        "if grep -qE '^\\s*#?\\s*{key}\\s*=' '{file}'; then \
           sed -i 's|^\\s*#\\?\\s*{key}\\s*=.*|{key} = \"{value}\"|' '{file}'; \
         else \
           echo '{key} = \"{value}\"' >> '{file}'; \
         fi",
        key = key, value = value, file = file,
    )
}

fn retroarch_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/retroarch", home);
    let cfg = format!("{}/retroarch.cfg", cfg_dir);

    let sets = [
        ("system_directory", format!("{}/bios", base)),
        ("savefile_directory", format!("{}/saves/retroarch", base)),
        ("savestate_directory", format!("{}/states/retroarch", base)),
        ("screenshot_directory", format!("{}/screenshots", base)),
        ("rgui_browser_directory", format!("{}/roms", base)),
        ("content_directory", format!("{}/roms", base)),
    ];

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));
    for (key, val) in &sets {
        parts.push(ini_set(&cfg, key, val));
    }
    parts.join(" && ")
}

fn duckstation_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.local/share/duckstation", home);
    let cfg = format!("{}/settings.ini", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // Ensure [Main] section exists
    parts.push(format!(
        "grep -q '^\\[Main\\]' '{}' || echo '[Main]' >> '{}'", cfg, cfg
    ));

    // BIOS path — goes under [Main]
    parts.push(format!(
        "if grep -q '^BIOSDirectory' '{cfg}'; then \
           sed -i 's|^BIOSDirectory.*|BIOSDirectory = {base}/bios|' '{cfg}'; \
         else \
           sed -i '/^\\[Main\\]/a BIOSDirectory = {base}/bios' '{cfg}'; \
         fi",
        cfg = cfg, base = base,
    ));

    // Ensure [MemoryCards] section
    parts.push(format!(
        "grep -q '^\\[MemoryCards\\]' '{}' || echo '[MemoryCards]' >> '{}'", cfg, cfg
    ));
    parts.push(format!(
        "if grep -q '^Directory' '{cfg}'; then \
           sed -i 's|^Directory.*|Directory = {base}/saves/duckstation|' '{cfg}'; \
         else \
           sed -i '/^\\[MemoryCards\\]/a Directory = {base}/saves/duckstation' '{cfg}'; \
         fi",
        cfg = cfg, base = base,
    ));

    // Ensure [GameList] section with ROM path
    parts.push(format!(
        "grep -q '^\\[GameList\\]' '{}' || echo '[GameList]' >> '{}'", cfg, cfg
    ));
    parts.push(format!(
        "if grep -q '^RecursivePaths' '{cfg}'; then \
           sed -i 's|^RecursivePaths.*|RecursivePaths = {base}/roms/ps1|' '{cfg}'; \
         else \
           sed -i '/^\\[GameList\\]/a RecursivePaths = {base}/roms/ps1' '{cfg}'; \
         fi",
        cfg = cfg, base = base,
    ));

    // Symlink savestates
    parts.push(format!(
        "[ -L '{base}/states/duckstation' ] || \
         (rm -rf '{base}/states/duckstation' && \
          mkdir -p '{cfg_dir}/savestates' && \
          ln -sfn '{cfg_dir}/savestates' '{base}/states/duckstation')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn pcsx2_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/PCSX2/inis", home);
    let cfg = format!("{}/PCSX2.ini", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // Ensure [Folders] section
    parts.push(format!(
        "grep -q '^\\[Folders\\]' '{}' || echo '[Folders]' >> '{}'", cfg, cfg
    ));

    let folders = [
        ("Bios", format!("{}/bios", base)),
        ("Savestates", format!("{}/states/pcsx2", base)),
        ("MemoryCards", format!("{}/saves/pcsx2", base)),
        ("Screenshots", format!("{}/screenshots", base)),
    ];

    for (key, val) in &folders {
        parts.push(format!(
            "if grep -q '^{key}' '{cfg}'; then \
               sed -i 's|^{key}.*|{key} = {val}|' '{cfg}'; \
             else \
               sed -i '/^\\[Folders\\]/a {key} = {val}' '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    // Ensure [GameList] with ROM path
    parts.push(format!(
        "grep -q '^\\[GameList\\]' '{}' || echo '[GameList]' >> '{}'", cfg, cfg
    ));
    parts.push(format!(
        "if grep -q '^RecursivePaths' '{cfg}'; then \
           sed -i 's|^RecursivePaths.*|RecursivePaths = {base}/roms/ps2|' '{cfg}'; \
         else \
           sed -i '/^\\[GameList\\]/a RecursivePaths = {base}/roms/ps2' '{cfg}'; \
         fi",
        cfg = cfg, base = base,
    ));

    parts.join(" && ")
}

fn dolphin_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/dolphin-emu", home);
    let cfg = format!("{}/Dolphin.ini", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // Ensure [General] section
    parts.push(format!(
        "grep -q '^\\[General\\]' '{}' || echo '[General]' >> '{}'", cfg, cfg
    ));

    // Set ROM search paths (GC + Wii)
    let iso_settings = [
        ("ISOPaths", "2"),
        ("ISOPath0", &format!("{}/roms/gc", base)),
        ("ISOPath1", &format!("{}/roms/wii", base)),
    ];

    for (key, val) in &iso_settings {
        parts.push(format!(
            "if grep -q '^{key}' '{cfg}'; then \
               sed -i 's|^{key}.*|{key} = {val}|' '{cfg}'; \
             else \
               sed -i '/^\\[General\\]/a {key} = {val}' '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    // Symlink saves (GC memory cards)
    let data_dir = format!("{}/.local/share/dolphin-emu", home);
    parts.push(format!(
        "[ -L '{base}/saves/dolphin' ] || \
         (rm -rf '{base}/saves/dolphin' && \
          mkdir -p '{data_dir}/GC' && \
          ln -sfn '{data_dir}/GC' '{base}/saves/dolphin')",
        base = base, data_dir = data_dir,
    ));

    // Symlink states
    parts.push(format!(
        "[ -L '{base}/states/dolphin' ] || \
         (rm -rf '{base}/states/dolphin' && \
          mkdir -p '{data_dir}/StateSaves' && \
          ln -sfn '{data_dir}/StateSaves' '{base}/states/dolphin')",
        base = base, data_dir = data_dir,
    ));

    parts.join(" && ")
}

fn ppsspp_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/ppsspp/PSP/SYSTEM", home);
    let cfg = format!("{}/ppsspp.ini", cfg_dir);

    let mut parts = vec![
        format!("mkdir -p '{}'", cfg_dir),
        format!("mkdir -p '{}/.config/ppsspp/PSP/SAVEDATA'", home),
        format!("mkdir -p '{}/.config/ppsspp/PSP/PPSSPP_STATE'", home),
    ];
    parts.push(format!("touch '{}'", cfg));

    // Ensure [General] section
    parts.push(format!(
        "grep -q '^\\[General\\]' '{}' || echo '[General]' >> '{}'", cfg, cfg
    ));

    // Set the "CurrentDirectory" to our ROMs path so the browser opens there
    parts.push(format!(
        "if grep -q '^CurrentDirectory' '{cfg}'; then \
           sed -i 's|^CurrentDirectory.*|CurrentDirectory = {base}/roms/psp|' '{cfg}'; \
         else \
           sed -i '/^\\[General\\]/a CurrentDirectory = {base}/roms/psp' '{cfg}'; \
         fi",
        cfg = cfg, base = base,
    ));

    // Symlink save/state dirs into unified tree
    parts.push(format!(
        "[ -L '{base}/saves/ppsspp' ] || \
         (rm -rf '{base}/saves/ppsspp' && \
          ln -sfn '{home}/.config/ppsspp/PSP/SAVEDATA' '{base}/saves/ppsspp')",
        base = base, home = home,
    ));
    parts.push(format!(
        "[ -L '{base}/states/ppsspp' ] || \
         (rm -rf '{base}/states/ppsspp' && \
          ln -sfn '{home}/.config/ppsspp/PSP/PPSSPP_STATE' '{base}/states/ppsspp')",
        base = base, home = home,
    ));

    parts.join(" && ")
}

fn rpcs3_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/rpcs3", home);

    // RPCS3 uses a YAML vfs.yml for virtual filesystem paths.
    // We point /dev_hdd0/ at our saves dir and create a games symlink.
    let vfs = format!("{}/vfs.yml", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];

    // Create vfs.yml if missing, set /games/ to our rom path
    parts.push(format!(
        "if [ ! -f '{vfs}' ]; then \
           printf '%s\\n' \
             '/games/: {base}/roms/ps3/' \
             '$(EmulatorDir): {cfg_dir}/' \
             '/dev_hdd0/: {cfg_dir}/dev_hdd0/' \
             '/dev_hdd1/: {cfg_dir}/dev_hdd1/' \
             '/dev_flash/: {cfg_dir}/dev_flash/' \
             '/dev_flash2/: {cfg_dir}/dev_flash2/' \
             '/dev_flash3/: {cfg_dir}/dev_flash3/' \
             '/dev_bdvd/: ' \
             '/dev_usb000/: ' \
             > '{vfs}'; \
         else \
           if grep -q '^/games/:' '{vfs}'; then \
             sed -i 's|^/games/:.*|/games/: {base}/roms/ps3/|' '{vfs}'; \
           else \
             echo '/games/: {base}/roms/ps3/' >> '{vfs}'; \
           fi; \
         fi",
        vfs = vfs, base = base, cfg_dir = cfg_dir,
    ));

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/rpcs3' ] || \
         (rm -rf '{base}/saves/rpcs3' && \
          mkdir -p '{cfg_dir}/dev_hdd0/home/00000001/savedata' && \
          ln -sfn '{cfg_dir}/dev_hdd0/home/00000001/savedata' '{base}/saves/rpcs3')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn cemu_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.local/share/Cemu", home);
    let cfg = format!("{}/settings.xml", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];

    // If settings.xml doesn't exist, create a minimal one with our paths
    parts.push(format!(
        "if [ ! -f '{cfg}' ]; then \
           printf '%s\\n' \
             '<?xml version=\"1.0\" encoding=\"utf-8\"?>' \
             '<content>' \
             '  <GamePaths>' \
             '    <Entry>{base}/roms/wiiu</Entry>' \
             '  </GamePaths>' \
             '</content>' \
             > '{cfg}'; \
         else \
           if ! grep -q '{base}/roms/wiiu' '{cfg}'; then \
             sed -i 's|</GamePaths>|    <Entry>{base}/roms/wiiu</Entry>\\n  </GamePaths>|' '{cfg}'; \
           fi; \
         fi",
        cfg = cfg, base = base,
    ));

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/cemu' ] || \
         (rm -rf '{base}/saves/cemu' && \
          mkdir -p '{cfg_dir}/mlc01/usr/save' && \
          ln -sfn '{cfg_dir}/mlc01/usr/save' '{base}/saves/cemu')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn ryujinx_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/Ryujinx", home);
    let cfg = format!("{}/Config.json", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];

    // If Config.json doesn't exist yet, create a minimal one with the game dirs path.
    // If it does exist, patch the game_dirs array using a python one-liner (jq not
    // always available, python3 always is on Arch).
    let rom_path = format!("{}/roms/switch", base);
    parts.push(format!(
        "if [ ! -f '{cfg}' ]; then \
           printf '%s\\n' '{{' \
             '  \"game_dirs\": [\"{rom_path}\"]' \
             '}}' > '{cfg}'; \
         else \
           python3 -c \"\
import json, sys; \
p='{cfg}'; f=open(p); c=json.load(f); f.close(); \
d=c.get('game_dirs',[]); \
r='{rom_path}'; \
changed=False; \
if r not in d: d.append(r); c['game_dirs']=d; changed=True; \
if changed: f=open(p,'w'); json.dump(c,f,indent=2); f.close(); \
\"; \
         fi",
        cfg = cfg, rom_path = rom_path,
    ));

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/ryujinx' ] || \
         (rm -rf '{base}/saves/ryujinx' && \
          mkdir -p '{cfg_dir}/bis/user/save' && \
          ln -sfn '{cfg_dir}/bis/user/save' '{base}/saves/ryujinx')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn xemu_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.local/share/xemu/xemu", home);
    let cfg = format!("{}/xemu.toml", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // Ensure [sys.files] section with eeprom and hdd image paths
    parts.push(format!(
        "grep -q '^\\[sys.files\\]' '{cfg}' || echo '[sys.files]' >> '{cfg}'",
        cfg = cfg,
    ));

    // Point BIOS (mcpx/flash) at our bios dir
    let bios_keys = [
        ("bootrom_path", format!("'{}/bios/mcpx_1.0.bin'", base)),
        ("flashrom_path", format!("'{}/bios/Complex_4627v1.03.bin'", base)),
    ];

    for (key, val) in &bios_keys {
        parts.push(format!(
            "if grep -q '^{key}' '{cfg}'; then \
               sed -i \"s|^{key}.*|{key} = {val}|\" '{cfg}'; \
             else \
               sed -i '/^\\[sys.files\\]/a {key} = {val}' '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/xemu' ] || \
         (rm -rf '{base}/saves/xemu' && \
          mkdir -p '{cfg_dir}' && \
          ln -sfn '{cfg_dir}' '{base}/saves/xemu')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn flycast_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.local/share/flycast", home);
    let cfg = format!("{}/emu.cfg", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // Ensure [config] section
    parts.push(format!(
        "grep -q '^\\[config\\]' '{}' || echo '[config]' >> '{}'", cfg, cfg
    ));

    let settings = [
        ("Dreamcast.ContentPath", format!("{}/roms/dc", base)),
    ];

    for (key, val) in &settings {
        parts.push(format!(
            "if grep -q '^{key}' '{cfg}'; then \
               sed -i 's|^{key}.*|{key} = {val}|' '{cfg}'; \
             else \
               sed -i '/^\\[config\\]/a {key} = {val}' '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/flycast' ] || \
         (rm -rf '{base}/saves/flycast' && \
          mkdir -p '{cfg_dir}/data' && \
          ln -sfn '{cfg_dir}/data' '{base}/saves/flycast')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn mame_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.mame", home);
    let cfg = format!("{}/mame.ini", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // MAME uses a space-separated ini format: key  value
    let settings = [
        ("rompath", format!("{}/roms/arcade", base)),
        ("snapshot_directory", format!("{}/screenshots", base)),
    ];

    for (key, val) in &settings {
        parts.push(format!(
            "if grep -q '^{key}' '{cfg}'; then \
               sed -i 's|^{key}.*|{key}                    {val}|' '{cfg}'; \
             else \
               echo '{key}                    {val}' >> '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/mame' ] || \
         (rm -rf '{base}/saves/mame' && \
          mkdir -p '{cfg_dir}/nvram' && \
          ln -sfn '{cfg_dir}/nvram' '{base}/saves/mame')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn shadps4_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.local/share/shadps4", home);
    let cfg = format!("{}/config.toml", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // Set game install/search directory
    let settings = [
        ("gameInstallDir", format!("{}/roms/ps4", base)),
    ];

    for (key, val) in &settings {
        parts.push(format!(
            "if grep -q '^{key}' '{cfg}'; then \
               sed -i 's|^{key}.*|{key} = \"{val}\"|' '{cfg}'; \
             else \
               echo '{key} = \"{val}\"' >> '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/shadps4' ] || \
         (rm -rf '{base}/saves/shadps4' && \
          mkdir -p '{cfg_dir}/savedata' && \
          ln -sfn '{cfg_dir}/savedata' '{base}/saves/shadps4')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn mgba_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/mgba", home);
    let cfg = format!("{}/config.ini", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // Ensure [ports.qt] section (mGBA-qt config section)
    parts.push(format!(
        "grep -q '^\\[ports.qt\\]' '{}' || echo '[ports.qt]' >> '{}'", cfg, cfg
    ));

    let settings = [
        ("savegamePath", format!("{}/saves/mgba", base)),
        ("savestatePath", format!("{}/states/mgba", base)),
        ("screenshotPath", format!("{}/screenshots", base)),
        ("lastDirectory", format!("{}/roms/gba", base)),
    ];

    for (key, val) in &settings {
        parts.push(format!(
            "if grep -q '^{key}' '{cfg}'; then \
               sed -i 's|^{key}.*|{key}={val}|' '{cfg}'; \
             else \
               sed -i '/^\\[ports.qt\\]/a {key}={val}' '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    // Also set BIOS path under [ports.qt] for GBA BIOS
    parts.push(format!(
        "if grep -q '^biosPath' '{cfg}'; then \
           sed -i 's|^biosPath.*|biosPath={base}/bios|' '{cfg}'; \
         else \
           sed -i '/^\\[ports.qt\\]/a biosPath={base}/bios' '{cfg}'; \
         fi",
        cfg = cfg, base = base,
    ));

    parts.join(" && ")
}

fn melonds_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/melonDS", home);
    let cfg = format!("{}/melonDS.ini", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    // melonDS uses a flat INI (no sections) for most path settings
    let settings = [
        ("LastROMFolder", format!("{}/roms/nds", base)),
        ("BIOS9Path", format!("{}/bios/bios9.bin", base)),
        ("BIOS7Path", format!("{}/bios/bios7.bin", base)),
        ("FirmwarePath", format!("{}/bios/firmware.bin", base)),
        ("SaveFilePath", format!("{}/saves/melonds", base)),
        ("SavestatePath", format!("{}/states/melonds", base)),
        ("ScreenshotPath", format!("{}/screenshots", base)),
    ];

    for (key, val) in &settings {
        parts.push(format!(
            "if grep -q '^{key}=' '{cfg}'; then \
               sed -i 's|^{key}=.*|{key}={val}|' '{cfg}'; \
             else \
               echo '{key}={val}' >> '{cfg}'; \
             fi",
            key = key, val = val, cfg = cfg,
        ));
    }

    parts.join(" && ")
}

fn vita3k_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.local/share/Vita3K/Vita3K", home);

    // Vita3K uses a config.yml — patch pref-path
    let cfg = format!("{}/config.yml", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];
    parts.push(format!("touch '{}'", cfg));

    parts.push(format!(
        "if grep -q '^pref-path:' '{cfg}'; then \
           sed -i 's|^pref-path:.*|pref-path: {cfg_dir}|' '{cfg}'; \
         else \
           echo 'pref-path: {cfg_dir}' >> '{cfg}'; \
         fi",
        cfg = cfg, cfg_dir = cfg_dir,
    ));

    // Symlink saves
    parts.push(format!(
        "[ -L '{base}/saves/vita3k' ] || \
         (rm -rf '{base}/saves/vita3k' && \
          mkdir -p '{cfg_dir}/ux0/user/00/savedata' && \
          ln -sfn '{cfg_dir}/ux0/user/00/savedata' '{base}/saves/vita3k')",
        base = base, cfg_dir = cfg_dir,
    ));

    parts.join(" && ")
}

fn esde_config_script(home: &str) -> String {
    let base = format!("{}/Emulation", home);
    let cfg_dir = format!("{}/.config/ES-DE", home);
    let cfg = format!("{}/es_settings.xml", cfg_dir);

    let mut parts = vec![format!("mkdir -p '{}'", cfg_dir)];

    // If es_settings.xml doesn't exist, create a minimal one with our ROM path.
    // ES-DE uses XML-style <string name="key" value="val" /> entries.
    parts.push(format!(
        "if [ ! -f '{cfg}' ]; then \
           printf '%s\\n' \
             '<?xml version=\"1.0\"?>' \
             '<config>' \
             '  <string name=\"ROMDirectory\" value=\"{base}/roms\" />' \
             '  <string name=\"MediaDirectory\" value=\"{base}/storage/es-de/media\" />' \
             '</config>' \
             > '{cfg}'; \
         else \
           if grep -q 'name=\"ROMDirectory\"' '{cfg}'; then \
             sed -i 's|name=\"ROMDirectory\" value=\"[^\"]*\"|name=\"ROMDirectory\" value=\"{base}/roms\"|' '{cfg}'; \
           else \
             sed -i 's|</config>|  <string name=\"ROMDirectory\" value=\"{base}/roms\" />\\n</config>|' '{cfg}'; \
           fi; \
           if grep -q 'name=\"MediaDirectory\"' '{cfg}'; then \
             sed -i 's|name=\"MediaDirectory\" value=\"[^\"]*\"|name=\"MediaDirectory\" value=\"{base}/storage/es-de/media\"|' '{cfg}'; \
           else \
             sed -i 's|</config>|  <string name=\"MediaDirectory\" value=\"{base}/storage/es-de/media\" />\\n</config>|' '{cfg}'; \
           fi; \
         fi",
        cfg = cfg, base = base,
    ));

    // Create the media directory for scraped artwork
    parts.push(format!("mkdir -p '{}/storage/es-de/media'", base));

    parts.join(" && ")
}

// ── Public setup ─────────────────────────────────────────────────────────────

/// Set up all button handlers for the emulators page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_filesystem(page_builder, window);
    setup_esde(page_builder, window);
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

// ── Filesystem Setup ─────────────────────────────────────────────────────────

fn setup_filesystem(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_emu_filesystem");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Emulators: Setup Filesystem button clicked");
        let window_ref = window.upcast_ref();

        let home = crate::config::env::get().home.clone();

        // Detect which emulators are already installed so we can offer to
        // configure them.
        let ra_installed = core::is_package_installed("retroarch");
        let duck_installed = core::is_package_installed("duckstation-gpl");
        let pcsx2_installed = core::is_package_installed("pcsx2-git");
        let dolphin_installed = core::is_package_installed("dolphin-emu");
        let ppsspp_installed = core::is_package_installed("ppsspp");
        let rpcs3_installed = core::is_package_installed("rpcs3-git");
        let cemu_installed = core::is_package_installed("cemu-git");
        let ryujinx_installed = core::is_package_installed("ryujinx");
        let xemu_installed = core::is_package_installed("xemu-git");
        let flycast_installed = core::is_package_installed("flycast");
        let mame_installed = core::is_package_installed("mame");
        let shadps4_installed = core::is_package_installed("shadps4");
        let vita3k_installed = core::is_package_installed("vita3k-git");
        let mgba_installed = core::is_package_installed("mgba-qt");
        let melonds_installed = core::is_package_installed("melonds-git");
        let esde_installed = core::is_package_installed("emulationstation-de");

        let any_installed = ra_installed || duck_installed || pcsx2_installed
            || dolphin_installed || ppsspp_installed || rpcs3_installed
            || cemu_installed || ryujinx_installed || xemu_installed
            || flycast_installed || mame_installed || shadps4_installed
            || vita3k_installed || mgba_installed || melonds_installed
            || esde_installed;

        let mut config = SelectionDialogConfig::new(
            "Emulation Filesystem Setup",
            "Creates ~/Emulation/ with ROM, BIOS, saves, states, and screenshot directories.\n\
             Optionally configure installed emulators to use this filesystem.",
        )
        .selection_type(SelectionType::Multi)
        .selection_required(false)
        .confirm_label("Setup");

        if any_installed {
            if ra_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_retroarch", "Configure RetroArch",
                    "Point BIOS, saves, states, screenshots, and ROM browser at ~/Emulation",
                    false,
                ));
            }
            if duck_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_duckstation", "Configure DuckStation",
                    "Point BIOS, memory cards, and game list at ~/Emulation",
                    false,
                ));
            }
            if pcsx2_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_pcsx2", "Configure PCSX2",
                    "Point BIOS, saves, states, and game list at ~/Emulation",
                    false,
                ));
            }
            if dolphin_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_dolphin", "Configure Dolphin",
                    "Add GC and Wii ROM paths from ~/Emulation",
                    false,
                ));
            }
            if ppsspp_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_ppsspp", "Configure PPSSPP",
                    "Set ROM browser + symlink saves/states into ~/Emulation",
                    false,
                ));
            }
            if rpcs3_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_rpcs3", "Configure RPCS3",
                    "Set /games/ virtual path and symlink savedata into ~/Emulation",
                    false,
                ));
            }
            if cemu_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_cemu", "Configure Cemu",
                    "Add Wii U ROM path and symlink saves into ~/Emulation",
                    false,
                ));
            }
            if ryujinx_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_ryujinx", "Configure Ryujinx",
                    "Add Switch ROM path and symlink saves into ~/Emulation",
                    false,
                ));
            }
            if xemu_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_xemu", "Configure xemu",
                    "Point BIOS paths at ~/Emulation/bios",
                    false,
                ));
            }
            if flycast_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_flycast", "Configure Flycast",
                    "Set Dreamcast ROM path and symlink saves into ~/Emulation",
                    false,
                ));
            }
            if mame_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_mame", "Configure MAME",
                    "Set arcade ROM path and screenshot directory",
                    false,
                ));
            }
            if shadps4_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_shadps4", "Configure ShadPS4",
                    "Set PS4 game directory and symlink saves into ~/Emulation",
                    false,
                ));
            }
            if vita3k_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_vita3k", "Configure Vita3K",
                    "Symlink savedata into ~/Emulation",
                    false,
                ));
            }
            if mgba_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_mgba", "Configure mGBA",
                    "Point saves, states, screenshots, and ROM browser at ~/Emulation",
                    false,
                ));
            }
            if melonds_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_melonds", "Configure melonDS",
                    "Point BIOS, saves, states, screenshots, and ROM folder at ~/Emulation",
                    false,
                ));
            }
            if esde_installed {
                config = config.add_option(SelectionOption::new(
                    "cfg_esde", "Configure ES-DE",
                    "Point ROM directory and media scraper at ~/Emulation",
                    false,
                ));
            }
        }

        let window_for_closure = window.clone();
        show_selection_dialog(window_ref, config, move |selected_ids| {
            let mut commands = CommandSequence::new();

            // Always create the directory tree
            commands = commands.then(
                Command::builder()
                    .normal()
                    .program("sh")
                    .args(&["-c", &build_mkdir_script(&home)])
                    .description("Creating ~/Emulation directory tree...")
                    .build(),
            );

            // Config patching for each selected emulator
            let cfg_map: &[(&str, fn(&str) -> String)] = &[
                ("cfg_retroarch", retroarch_config_script as fn(&str) -> String),
                ("cfg_duckstation", duckstation_config_script),
                ("cfg_pcsx2", pcsx2_config_script),
                ("cfg_dolphin", dolphin_config_script),
                ("cfg_ppsspp", ppsspp_config_script),
                ("cfg_rpcs3", rpcs3_config_script),
                ("cfg_cemu", cemu_config_script),
                ("cfg_ryujinx", ryujinx_config_script),
                ("cfg_xemu", xemu_config_script),
                ("cfg_flycast", flycast_config_script),
                ("cfg_mame", mame_config_script),
                ("cfg_shadps4", shadps4_config_script),
                ("cfg_vita3k", vita3k_config_script),
                ("cfg_mgba", mgba_config_script),
                ("cfg_melonds", melonds_config_script),
                ("cfg_esde", esde_config_script),
            ];

            for (id, script_fn) in cfg_map {
                if selected_ids.iter().any(|s| s == *id) {
                    let emu_name = id.strip_prefix("cfg_").unwrap_or(id);
                    commands = commands.then(
                        Command::builder()
                            .normal()
                            .program("sh")
                            .args(&["-c", &script_fn(&home)])
                            .description(&format!("Configuring {} for ~/Emulation...", emu_name))
                            .build(),
                    );
                }
            }

            task_runner::run(
                window_for_closure.upcast_ref(),
                commands.build(),
                "Emulation Filesystem Setup",
            );
        });
    });
}

// ── ES-DE Frontend ──────────────────────────────────────────────────────────

fn setup_esde(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_emu_esde");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Emulators: ES-DE Frontend button clicked");

        let home = crate::config::env::get().home.clone();
        let mut commands = CommandSequence::new();

        // Ensure ~/Emulation tree exists first
        commands = commands.then(
            Command::builder()
                .normal()
                .program("sh")
                .args(&["-c", &build_mkdir_script(&home)])
                .description("Ensuring ~/Emulation directory tree exists...")
                .build(),
        );

        // ES-DE is in the AUR as emulationstation-de
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "emulationstation-de"])
                .description("Installing ES-DE from AUR...")
                .build(),
        );

        // Configure ES-DE to use ~/Emulation paths
        commands = commands.then(
            Command::builder()
                .normal()
                .program("sh")
                .args(&["-c", &esde_config_script(&home)])
                .description("Configuring ES-DE for ~/Emulation paths...")
                .build(),
        );

        task_runner::run(
            window.upcast_ref(),
            commands.build(),
            "ES-DE Frontend Installation",
        );
    });
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
            "RetroArch will be installed with assets and configured for ~/Emulation.\n\
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
            let home = crate::config::env::get().home.clone();
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

            // Post-install: configure RetroArch for ~/Emulation paths
            commands = commands.then(
                Command::builder()
                    .normal()
                    .program("sh")
                    .args(&["-c", &retroarch_config_script(&home)])
                    .description("Configuring RetroArch for ~/Emulation paths...")
                    .build(),
            );

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

    fn config_script(&self, home: &str) -> String {
        match self {
            Self::DuckStation => duckstation_config_script(home),
            Self::Pcsx2 => pcsx2_config_script(home),
            Self::Rpcs3 => rpcs3_config_script(home),
            Self::ShadPs4 => shadps4_config_script(home),
            Self::Ppsspp => ppsspp_config_script(home),
            Self::Vita3k => vita3k_config_script(home),
            Self::Mgba => mgba_config_script(home),
            Self::Melonds => melonds_config_script(home),
            Self::Dolphin => dolphin_config_script(home),
            Self::Cemu => cemu_config_script(home),
            Self::Ryujinx => ryujinx_config_script(home),
            Self::Xemu => xemu_config_script(home),
            Self::Flycast => flycast_config_script(home),
            Self::Mame => mame_config_script(home),
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

        let home = crate::config::env::get().home.clone();
        let mut commands = CommandSequence::new();

        // Ensure ~/Emulation tree exists first
        commands = commands.then(
            Command::builder()
                .normal()
                .program("sh")
                .args(&["-c", &build_mkdir_script(&home)])
                .description("Ensuring ~/Emulation directory tree exists...")
                .build(),
        );

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

        // Post-install: configure for ~/Emulation paths
        commands = commands.then(
            Command::builder()
                .normal()
                .program("sh")
                .args(&["-c", &emu.config_script(&home)])
                .description(&format!("Configuring {} for ~/Emulation paths...", label))
                .build(),
        );

        task_runner::run(
            window.upcast_ref(),
            commands.build(),
            &format!("{} Installation", label),
        );
    });
}
