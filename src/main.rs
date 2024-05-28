#![feature(panic_info_message)]

use std::{process, thread};

use crate::service::ServiceManager;

mod commands;
mod config;
mod ipc;
mod panic;
mod service;

fn main() -> ! {
    if process::id() != 1 {
        println!("Init must be ran as PID 1");
        process::exit(1);
    }

    panic::init_handler();

    let init_path = config::read_config().init_path;
    if !init_path.exists() {
        panic!("Init config does not exist.");
    }

    thread::spawn(ipc::init);

    let mut service_manager = ServiceManager::new(init_path.join("services"));
    thread::spawn(move || service_manager.load_all());

    init::infinite_loop();
}
