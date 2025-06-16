#![feature(never_type, panic_payload_as_str)]

use panic::crash_log;
use prelude::logger::{self, fatal, info};
use std::{
    env,
    path::PathBuf,
    process::{self, Command},
};

mod commands;
mod config;
mod mount;
mod panic;

#[prelude::entry(ok: !)]
fn main() {
    // FIXME: Signal the actual init.
    if process::id() != 1 {
        let exe = PathBuf::from(env::args().next().unwrap_or_default())
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // FIXME: Forward this to the actual init process with a signal or similar otherwise the user running the command will be halting the system instead.
        match exe.as_str() {
            "poweroff" => commands::poweroff(),
            "reboot" => commands::reboot(),
            _ => {
                fatal!("Init must be ran as PID 1");
                process::exit(1);
            }
        }
    }

    panic::init_handler();

    logger::set_app_name!();

    let system_config = match config::read() {
        Ok(config) => config,
        Err(error) => {
            crash_log(Some(*error), None);
        }
    };

    // Initialize environment variables
    unsafe {
        env::remove_var("BOOT_IMAGE");
        // Compatibility with the XDG config path
        // TODO: Set this per-user at login
        env::set_var("XDG_CONFIG_HOME", "/config");

        for (key, value) in system_config.environment {
            env::set_var(key, value);
        }
    }

    let mounts = [
        ("proc", "/proc"),
        ("sysfs", "/sys"),
        ("tmpfs", "/tmp"),
        ("devtmpfs", "/dev"),
    ];

    for (fs_type, target) in mounts {
        mount::pseudofs(fs_type, target);
    }

    info!("Starting service daemon");
    Command::new("/bin/serviced")
        .spawn()
        .expect("Failed to start service daemon")
        .wait()
        .expect("Service daemon exited with non-zero status code");

    panic!("Service daemon exited with non-zero status code")
}
