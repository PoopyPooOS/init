use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    PowerOff,
    Reboot,
    /// id of the service
    ServiceReady(String),
}
