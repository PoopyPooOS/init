use logger::fatal;
use std::{fs, path::PathBuf, process::Command};

pub fn pseudofs(fs_type: &str, target: &str) {
    if !PathBuf::from(target).exists() {
        fs::create_dir_all(target).expect("Failed to create pseudofs mount directory");
    }

    let mut command = Command::new("/system/bin/mount");
    command.args(["-t", fs_type, target, target]);

    match command.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(_) => {}
            Err(error) => {
                fatal!(format!("Failed to mount pseudofs '{fs_type}' to '{target}': {error}"));
                panic!("Failed to mount pseudofs");
            }
        },
        Err(error) => {
            fatal!(format!("Failed to mount pseudofs '{fs_type}' to '{target}': {error}"));
            panic!("Failed to mount pseudofs");
        }
    }
}
