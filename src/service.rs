use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use colored::Colorize;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServiceManagerConfig {
    service: Vec<Service>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    pub path: String,
    pub id: String,
    #[serde(default)]
    pub dependencies: Option<Vec<String>>,
    #[serde(default)]
    pub io: Option<Vec<IoOption>>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum IoOption {
    Out,
    In,
    Err,
}

#[derive(Debug, Clone)]
pub struct ServiceManager {
    pub running_services: Vec<Service>,
    pub services: Vec<Service>,
    service_path: PathBuf,
}

impl ServiceManager {
    pub fn new(service_path: PathBuf) -> Self {
        if !service_path.exists() {
            panic!("Services config not found.");
        }

        let services = {
            let config_raw = fs::read_to_string(service_path.join("services.toml"))
                .expect("Failed to read services file (/etc/init/services/services.toml)");

            toml::from_str::<ServiceManagerConfig>(&config_raw)
                .expect("Failed to parse services file (/etc/init/services/services.toml)")
                .service
        };

        Self {
            running_services: Vec::new(),
            services,
            service_path,
        }
    }

    pub fn load_all(&mut self) -> ! {
        println!("Loading services...\n");

        let shared_self = Arc::new(Mutex::new(self.clone()));
        let mut handles = vec![];

        for service in self.services.clone() {
            let shared_self_clone = Arc::clone(&shared_self);
            let handle = thread::spawn(move || {
                let mut locked_self = shared_self_clone.lock().unwrap();
                while !locked_self.can_start(&service) {
                    drop(locked_self);
                    thread::sleep(Duration::from_millis(100));
                    locked_self = shared_self_clone.lock().unwrap();
                }
                if locked_self.start_service(&service).is_ok() {
                    println!("[  {}  ] {}", "OK".green(), service.name);
                    locked_self.running_services.push(service.clone());
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        init::infinite_loop()
    }

    fn can_start(&self, service: &Service) -> bool {
        if let Some(dependencies) = &service.dependencies {
            for dependency in dependencies {
                if dependency == "*" {
                    return self.running_services.len() == self.services.len() - 1;
                }
                if !self.running_services.iter().any(|s| &s.id == dependency) {
                    return false;
                }
            }
        }

        true
    }

    pub fn start_service(&mut self, service: &Service) -> Result<(), ()> {
        let sub_services = fs::read_dir(self.service_path.join(&service.path)).unwrap();

        for sub_service in sub_services {
            if sub_service.is_err() {
                return Ok(());
            }

            let sub_service = sub_service.unwrap();
            let path = sub_service.path();

            if path.is_dir() {
                continue;
            }

            let shell_script = path.file_name().unwrap().to_str().unwrap().ends_with(".sh");

            if shell_script {
                let mut command = Command::new("/sbin/shell");

                if let Some(io) = &service.io {
                    for option in io {
                        match option {
                            IoOption::Out => command.stdout(Stdio::inherit()),
                            IoOption::In => command.stdin(Stdio::inherit()),
                            IoOption::Err => command.stderr(Stdio::inherit()),
                        };
                    }
                }

                command.arg(&path);
                #[allow(clippy::expect_fun_call)]
                command
                    .spawn()
                    .expect(&format!("Failed to start sub-service from service: {}", service.name));
            } else {
                let mut command = Command::new(&path);

                if let Some(io) = &service.io {
                    for option in io {
                        match option {
                            IoOption::Out => command.stdout(Stdio::inherit()),
                            IoOption::In => command.stdin(Stdio::inherit()),
                            IoOption::Err => command.stderr(Stdio::inherit()),
                        };
                    }
                }

                #[allow(clippy::expect_fun_call)]
                command
                    .spawn()
                    .expect(&format!("Failed to start sub-service from service: {}", service.name));
            }
        }

        Ok(())
    }
}
