use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    PowerOff,
    Reboot,
}
