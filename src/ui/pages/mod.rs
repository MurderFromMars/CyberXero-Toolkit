//! Page-specific button handlers and logic.
//!
//! This module organizes button handlers by page:
//! - `main_page`: System update, drivers, package managers
//! - `gaming_tools`: Steam, controllers, game launchers
//! - `containers_vms`: Docker, Podman, VirtualBox, KVM
//! - `multimedia_tools`: OBS, Jellyfin
//! - `customization`: ZSH, themes, wallpapers
//! - `servicing`: System fixes and maintenance

pub mod containers_vms;
pub mod customization;
pub mod gaming_tools;
pub mod main_page;
pub mod multimedia_tools;
pub mod servicing;

