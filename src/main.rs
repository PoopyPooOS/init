#![feature(never_type, panic_payload_as_str)]

use logger::{fatal, info};
use panic::crash_log;
use std::{
    env,
    error::Error,
    path::PathBuf,
    process::{self, Command},
};

mod commands;
mod config;
mod mount;
mod panic;

fn main() -> Result<!, Box<dyn Error>> {
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

    // // Temporary fix for certain TUI apps that use /dev/tty
    // (|| -> Result<(), io::Error> {
    //     fs::remove_file("/dev/tty")?;
    //     unix::fs::symlink("/dev/console", "/dev/tty")?;

    //     Ok(())
    // })()
    // .unwrap_or_else(|err| {
    //     warn!(format!(
    //         "Failed to create symlink from /dev/console to /dev/tty: {err}"
    //     ));
    // });

    info!("Starting service daemon");
    Command::new("/system/bin/serviced")
        .spawn()
        .expect("Failed to start service daemon")
        .wait()
        .expect("Service daemon exited with non-zero status code");

    panic!("Service daemon exited with non-zero status code");
}
