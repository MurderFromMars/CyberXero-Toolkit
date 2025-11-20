use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn detect_aur_helper() -> Option<&'static str> {
    const PRIORITY: [&str; 2] = ["paru", "yay"];

    for &cmd in PRIORITY.iter() {
        if is_executable_in_path(cmd) {
            return Some(cmd);
        }
    }

    None
}

fn is_executable_in_path(cmd: &str) -> bool {
    if cmd.contains(std::path::MAIN_SEPARATOR) {
        return PathBuf::from(cmd).is_file();
    }

    let paths = match env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };

    for dir in env::split_paths(&paths) {
        let mut candidate = dir.clone();
        candidate.push(cmd);
        if candidate.exists() {
            if let Ok(metadata) = std::fs::metadata(&candidate) {
                let perms = metadata.permissions();
                if perms.mode() & 0o111 != 0 {
                    return true;
                }
            }
        }
    }

    false
}
