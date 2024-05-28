use crossterm::{execute, terminal::{Clear, ClearType}, cursor::MoveTo};
use nix::sys::reboot::{reboot as linux_reboot, RebootMode};

pub fn poweroff() {
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear the terminal");
    println!("Exiting");
    linux_reboot(RebootMode::RB_POWER_OFF).expect("Failed to power off the system");
}

pub fn reboot() {
    execute!(std::io::stdout(), Clear(ClearType::All)).expect("Failed to clear the terminal");
    println!("Exiting");
    linux_reboot(RebootMode::RB_AUTOBOOT).expect("Failed to power off the system");
}