use std::fs;

use crate::commands;
use ipc_init::Command;
use linux_ipc::IpcChannel;

pub fn init() -> ! {
    fs::create_dir_all("/tmp/init/services").expect("Failed to setup directories for IPC");
    let mut ipc = IpcChannel::new("/tmp/init/init.sock").expect("Failed to create IPC channel");

    loop {
        let (command, _) = ipc.receive::<Command, ()>().expect("Failed to listen on IPC channel");

        match command {
            Command::PowerOff => commands::poweroff(),
            Command::Reboot => commands::reboot(),
        }
    }
}
