use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use logger::info;
use nix::sys::reboot::{reboot as linux_reboot, RebootMode};
use std::io::stdout;

pub fn poweroff() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear the terminal");
    info!("Shutting down system...");
    cleanup();
    linux_reboot(RebootMode::RB_POWER_OFF).expect("Failed to power off the system");
}

pub fn reboot() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear the terminal");
    info!("Rebooting system...");
    cleanup();
    linux_reboot(RebootMode::RB_AUTOBOOT).expect("Failed to power off the system");
}

pub fn cleanup() {
    // TODO: Clean up processes and other resources
}
