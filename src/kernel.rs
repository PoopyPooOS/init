use linux_args::KernelArgs;
use std::fs;

#[allow(dead_code)]
pub fn get_args() -> KernelArgs {
    let cmdline = fs::read_to_string("/proc/cmdline").unwrap_or_default();

    if cmdline.is_empty() {
        return KernelArgs::default();
    }

    KernelArgs::parse(&cmdline)
}
