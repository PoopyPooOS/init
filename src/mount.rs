use crate::panic::crash_log;
use prelude::logger::make_fatal;
use std::{fs, path::PathBuf, process::Command};

pub fn pseudofs(fs_type: &str, target: &str) {
    if !PathBuf::from(target).exists() {
        fs::create_dir_all(target).expect("Failed to create pseudofs mount directory");
    }

    let mut command = Command::new("/bin/mount");
    command.args(["-t", fs_type, target, target]);

    match command.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(_) => {}
            Err(err) => {
                crash_log(
                    Some(make_fatal!(
                        "Failed to mount pseudofs '{fs_type}' to '{target}': {err}"
                    )),
                    None,
                );
            }
        },
        Err(err) => {
            crash_log(
                Some(make_fatal!(
                    "Failed to mount pseudofs '{fs_type}' to '{target}': {err}"
                )),
                None,
            );
        }
    }
}
