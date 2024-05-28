use crate::commands;
use ipc_types::Command;
use linux_ipc::IpcChannel;

pub fn init() -> ! {
    let mut ipc = IpcChannel::new("/tmp/init.sock").expect("Failed to create IPC channel");

    loop {
        let command: Command = ipc.receive().expect("Failed to listen on IPC channel");

        match command {
            Command::PowerOff => commands::poweroff(),
            Command::Reboot => commands::reboot(),
        }
    }
}
