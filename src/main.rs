#![feature(never_type, panic_payload_as_str)]

use logger::{fatal, info};
use std::{
    env, fs, io,
    os::unix,
    path::PathBuf,
    process::{self, Command},
    thread,
};

mod commands;
mod config;
mod kernel;
mod mount;
mod panic;

#[tokio::main]
async fn main() -> Result<!, Box<dyn std::error::Error>> {
    if process::id() != 1 {
        let exe = PathBuf::from(env::args().next().unwrap_or_default())
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

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

    env::set_var("LOGGER_APP_NAME", env!("CARGO_PKG_NAME"));

    let system_config = match config::read().await {
        Ok(config) => config,
        Err(error) => {
            error.output();
            process::exit(1);
        }
    };

    // Initialize environment variables
    env::remove_var("BOOT_IMAGE");
    for (key, value) in system_config.environment {
        env::set_var(key, value);
    }

    let mount_threads = [
        thread::spawn(|| mount::pseudofs("proc", "/proc")),
        thread::spawn(|| mount::pseudofs("sysfs", "/sysfs")),
        thread::spawn(|| mount::pseudofs("tmpfs", "/tmp")),
        thread::spawn(|| mount::pseudofs("devtmpfs", "/dev")),
    ];

    for thread in mount_threads {
        thread.join().unwrap();
    }

    // Temporary fix for certain TUI apps that use /dev/tty
    (|| -> Result<(), io::Error> {
        fs::remove_file("/dev/tty")?;
        unix::fs::symlink("/dev/console", "/dev/tty")?;

        Ok(())
    })()
    .unwrap_or_else(|err| eprintln!("Failed to create symlink from /dev/console to /dev/tty: {err}"));

    info!("Starting service daemon");
    Command::new("/system/bin/serviced")
        .spawn()
        .expect("Failed to start service daemon")
        .wait()
        .expect("Service daemon exited with non-zero status code");

    panic!("Service daemon exited with non-zero status code");
}
