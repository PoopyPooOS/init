use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use nix::sys::reboot::{reboot as linux_reboot, RebootMode};

pub struct Commands;

#[allow(clippy::unused_self)]
impl Commands {
    pub fn poweroff(&self) {
        execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear the terminal");
        println!("Shutting down system...");
        linux_reboot(RebootMode::RB_POWER_OFF).expect("Failed to power off the system");
    }

    pub fn reboot(&self) {
        execute!(std::io::stdout(), Clear(ClearType::All)).expect("Failed to clear the terminal");
        println!("Rebooting system...");
        linux_reboot(RebootMode::RB_AUTOBOOT).expect("Failed to power off the system");
    }
}
