#![feature(panic_info_message)]

use colored::{Color, Colorize};
use std::{env, fs, io, os::unix, process, sync::Arc, thread};
use tokio::sync::{Mutex, RwLock};

use commands::Commands;
use ipc_init::Command;
use linux_ipc::IpcChannel;

mod commands;
mod config;
mod environment;
mod kernel;
mod mount;
mod panic;
mod service;

#[tokio::main]
async fn main() -> ! {
    if process::id() != 1 {
        eprintln!("Init must be ran as PID 1");
        process::exit(1);
    }

    panic::init_handler();

    let userspace_config = config::read();
    assert!(userspace_config.init_path.exists(), "Init config does not exist.");

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

    (|| -> Result<(), io::Error> {
        fs::remove_file("/dev/tty")?;
        unix::fs::symlink("/dev/console", "/dev/tty")?;

        Ok(())
    })()
    .unwrap_or_else(|err| eprintln!("Failed to create symlink from /dev/console to /dev/tty: {err}"));

    let mut service_manager = service::Manager::new(userspace_config.init_path.join("services"));
    let ready_services_list: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
    let list_reader = Arc::clone(&ready_services_list);
    let list_writer = Arc::clone(&ready_services_list);
    tokio::spawn(ipc_init(list_writer));

    service_manager.load_all(list_reader).await
}

async fn ipc_init(service_ready_list: Arc<RwLock<Vec<String>>>) -> ! {
    let mut service_ready_list = service_ready_list.write().await;
    tokio::fs::create_dir_all("/tmp/init/services")
        .await
        .expect("Failed to setup directories for IPC");

    let mut ipc = IpcChannel::new("/tmp/init/init.sock").expect("Failed to create IPC channel");

    loop {
        let (command, _) = ipc.receive::<Command, ()>().expect("Failed to listen on IPC channel");

        println!("{command:#?}");

        match command {
            Command::PowerOff => Commands.poweroff(),
            Command::Reboot => Commands.reboot(),
            Command::ServiceReady(id) => {
                println!("[  {}  ] Started {}", "OK".color(Color::Green), id);
                service_ready_list.push(id);
            }
        }
    }
}
