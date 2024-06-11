use linux_ipc::IpcChannel;
pub use types::*;

mod types;

pub struct Init {
    ipc: IpcChannel,
}

impl Init {
    pub fn new(socket_path: &str) -> Self {
        let ipc = IpcChannel::connect(socket_path).unwrap();

        Self { ipc }
    }

    pub fn reboot(&mut self) {
        self.ipc
            .send::<Command, ()>(Command::Reboot)
            .expect("Failed to send reboot command");
    }

    pub fn poweroff(&mut self) {
        self.ipc
            .send::<Command, ()>(Command::PowerOff)
            .expect("Failed to send poweroff command");
    }
}
