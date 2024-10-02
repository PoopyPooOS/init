#![feature(never_type)]

use commands::Commands;
use ipc_init::Command;
use linux_ipc::IpcChannel;
use std::{env, fs, io, os::unix, process, thread};

mod commands;
mod config;
mod environment;
mod kernel;
mod mount;
mod panic;

#[tokio::main]
async fn main() -> Result<!, Box<dyn std::error::Error>> {
    if process::id() != 1 {
        eprintln!("Init must be ran as PID 1");
        process::exit(1);
    }

    panic::init_handler();

    let userspace_config = config::read();
    // There is no init config yet (and probably wont be).
    // assert!(userspace_config.init_path.exists(), "Init config does not exist.");

    env::remove_var("BOOT_IMAGE");
    let env = environment::parse_environment_file(userspace_config.env_vars_path).expect("Failed to parse environment file.");

    for (key, value) in env {
        env::set_var(key, value);
    }

    let mount_threads = [
        thread::spawn(|| mount::pseudofs("proc", "/proc")),
        thread::spawn(|| mount::pseudofs("sysfs", "/sys")),
        thread::spawn(|| mount::pseudofs("tmpfs", "/tmp")),
        thread::spawn(|| mount::pseudofs("devtmpfs", "/dev")),
    ];

    for thread in mount_threads {
        thread.join().unwrap();
    }

    // Temporary fix for certain TUI apps that need /dev/tty
    (|| -> Result<(), io::Error> {
        fs::remove_file("/dev/tty")?;
        unix::fs::symlink("/dev/console", "/dev/tty")?;

        Ok(())
    })()
    .unwrap_or_else(|err| eprintln!("Failed to create symlink from /dev/console to /dev/tty: {err}"));

    process::Command::new("/sbin/serviced").spawn()?;

    ipc_init().await
}

async fn ipc_init() -> ! {
    tokio::fs::create_dir_all("/tmp/ipc")
        .await
        .expect("Failed to setup directories for IPC");

    let mut ipc = IpcChannel::new("/tmp/ipc/init.sock").expect("Failed to create IPC channel");

    loop {
        let (command, _) = ipc.receive::<Command, ()>().expect("Failed to listen on IPC channel");

        match command {
            Command::PowerOff => Commands.poweroff(),
            Command::Reboot => Commands.reboot(),
        }
    }
}
