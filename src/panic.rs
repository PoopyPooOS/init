use crate::commands;
use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use prelude::logger::{fatal, warn, Colorize, Log, LogLevel};
use rustix::system::{reboot, RebootCommand};
use std::{
    panic::PanicHookInfo,
    process::{Command, Stdio},
};

pub fn init_handler() {
    std::panic::set_hook(Box::new(|panic_info| crash_log(None, Some(panic_info))));
}

pub fn crash_log(log: Option<Log>, panic_info: Option<&PanicHookInfo>) -> ! {
    if let Some(panic_info) = panic_info {
        fatal!(
            "Panic occured in the init at {location}: \n{message}",
            location = panic_info.location().expect("Failed to get panic location"),
            message = panic_info
                .payload_as_str()
                .unwrap_or("Failed to get panic message")
        );
    } else if let Some(log) = log {
        Log {
            level: LogLevel::Fatal,
            message: format!("An error occured in the init: \n{}", log.message),
            location: log.location,
            hint: log.hint,
        }
        .output();
    }

    if let Err(err) = reboot(RebootCommand::CadOn) {
        warn!("Failed to enable Ctrl-Alt-Del keybind: {err}");
    }

    println!("\n{}", "Options:".bold());
    println!(" {}", "Press enter to try to spawn a shell.".bold());
    println!(" {}", "Press Ctrl-Alt-Del to reboot.".bold());

    let _ = enable_raw_mode();

    loop {
        if matches!(read().unwrap(), Event::Key(event) if event.code == KeyCode::Enter) {
            let _ = disable_raw_mode();

            let mut shell = Command::new("/system/bin/shell")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .env("PATH", "/bin")
                .spawn()
                .expect("Failed to start shell");

            let _ = shell.wait();
            commands::poweroff();
        }
    }
}
