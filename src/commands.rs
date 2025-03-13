use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use logger::info;
use rustix::system::{RebootCommand, reboot as linux_reboot};
use std::io::stdout;

pub fn poweroff() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear the terminal");
    info!("Shutting down system...");
    cleanup();
    linux_reboot(RebootCommand::PowerOff).expect("Failed to power off the system");
}

pub fn reboot() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear the terminal");
    info!("Rebooting system...");
    cleanup();
    linux_reboot(RebootCommand::Restart).expect("Failed to power off the system");
}

pub fn cleanup() {
    // TODO: Clean up processes and other resources
}
