use std::{fs, path::PathBuf, process::Command};

pub fn pseudofs(fs_type: &str, target: &str) {
    if !PathBuf::from(target).exists() {
        fs::create_dir_all(target).expect("Failed to create pseudofs mount directory");
    }

    Command::new("/sbin/mount")
        .args(["-t", fs_type])
        .args([target, target])
        .spawn()
        .expect("Failed to mount filesystem")
        .wait()
        .expect("Failed to wait for proess to exit");
}
