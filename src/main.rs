#![feature(panic_info_message)]

use std::{env, fs, io, os::unix, process, thread};

use crate::service::ServiceManager;

mod commands;
mod config;
mod environment;
mod ipc;
mod mount;
mod panic;
mod service;

fn main() -> ! {
    if process::id() != 1 {
        println!("Init must be ran as PID 1");
        process::exit(1);
    }

    panic::init_handler();

    let userspace_config = config::read_config();
    if !userspace_config.init_path.exists() {
        panic!("Init config does not exist.");
    }

    println!("Setting environment variables...");

    let env = environment::parse_environment_file(userspace_config.env_vars_path).expect("Failed to parse environment file.");

    for (key, value) in env {
        env::set_var(key, value);
    }

    println!("Mounting filesystems...");
    mount::mount_pseudofs("proc", "/proc");
    mount::mount_pseudofs("sysfs", "/sys");
    mount::mount_pseudofs("tmpfs", "/tmp");
    mount::mount_pseudofs("devtmpfs", "/dev");

    let result = (|| -> Result<(), io::Error> {
        fs::remove_file("/dev/tty")?;
        unix::fs::symlink("/dev/console", "/dev/tty")?;

        Ok(())
    })();

    result.unwrap_or_else(|err| eprintln!("Failed to create symlink from /dev/console to /dev/tty: {}", err));

    thread::spawn(ipc::init);

    let mut service_manager = ServiceManager::new(userspace_config.init_path.join("services"));
    thread::spawn(move || service_manager.load_all());

    // Command::new("/sbin/shell")
    //     .env("PATH", "/sbin:/bin")
    //     .env("HOME", "/root")
    //     .stdout(Stdio::inherit())
    //     .stderr(Stdio::inherit())
    //     .stdin(Stdio::inherit())
    //     .spawn()
    //     .expect("Failed to spawn shell");

    init::infinite_loop();
}
