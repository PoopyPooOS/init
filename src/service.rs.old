use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};

use colored::{Color, Colorize};
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub id: String,
    pub exec: Exec,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub dependencies: Option<Vec<String>>,
    #[serde(default)]
    pub io: Option<Vec<IoOption>>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct Exec(String);

impl Exec {
    pub fn as_command(&self) -> Command {
        let split = self.0.split_whitespace().collect::<Vec<&str>>();
        let path = split[0];
        let args = &split[1..];

        let mut command = Command::new(path);
        command.args(args);

        // By default, services are silent and can only log to files.
        command.stdout(Stdio::null());
        command.stdin(Stdio::null());
        command.stderr(Stdio::null());

        command
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum IoOption {
    Out,
    In,
    Err,
}

#[derive(Debug, Clone)]
pub struct Manager {
    pub services: Vec<Service>,
}

impl Manager {
    pub fn new(services_path: PathBuf) -> Self {
        assert!(services_path.exists(), "Services config not found.");

        let services = fs::read_dir(services_path)
            .expect("Failed to read service config directory")
            .filter_map(Result::ok)
            .filter(|file| file.path().extension().map_or(false, |ext| ext == "toml"))
            .map(|file| fs::read_to_string(file.path()).expect("Failed to read service file"))
            .map(|raw| toml::from_str::<Service>(&raw).expect("Failed to parse service file"))
            .filter(|service| service.enabled)
            .collect::<Vec<_>>();

        Self { services }
    }

    #[allow(clippy::unused_async)]
    pub async fn load_all(&mut self, service_ready_list: Arc<RwLock<Vec<String>>>) -> ! {
        assert!(!self.has_circular_dependency(), "Circular dependency detected!");

        for service in &self.services {
            println!("[  {}  ] Starting {}", "OK".color(Color::Green), service.name);

            loop {
                let service_ready_list = Arc::clone(&service_ready_list);
                let can_start = self.can_start(service, service_ready_list).await;
                // let service_name = &service.name;

                // println!("{:#?}", self.running_services);
                // println!("{service_name} can start: {can_start}");

                if can_start {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(20)).await;
            }

            self.start_service(service);
        }

        init::infinite_loop()
    }

    async fn can_start(&self, service: &Service, service_ready_list: Arc<RwLock<Vec<String>>>) -> bool {
        if let Some(dependencies) = &service.dependencies {
            for dependency in dependencies {
                let service_ready_list = service_ready_list.read().await;

                if dependency == "*" {
                    return service_ready_list.len() == self.services.len() - 1;
                }
                if !service_ready_list.iter().any(|id| id == dependency) {
                    return false;
                }
            }
        }

        true
    }

    #[allow(clippy::unused_self)]
    pub fn start_service(&self, service: &Service) {
        let mut command = service.exec.as_command();

        if let Some(io) = &service.io {
            for option in io {
                match option {
                    IoOption::Out => command.stdout(Stdio::inherit()),
                    IoOption::In => command.stdin(Stdio::inherit()),
                    IoOption::Err => command.stderr(Stdio::inherit()),
                };
            }
        }

        command
            .status()
            .unwrap_or_else(|_| panic!("Failed to start service \"{}\"", service.name));
    }

    fn has_circular_dependency(&self) -> bool {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut graph = HashMap::new();

        for service in &self.services {
            graph.insert(&service.id, &service.dependencies);
        }

        for service in &self.services {
            if self.is_cyclic(&service.id, &graph, &mut visited, &mut stack) {
                return true;
            }
        }

        false
    }

    #[allow(clippy::only_used_in_recursion)]
    fn is_cyclic(
        &self,
        node: &String,
        graph: &HashMap<&String, &Option<Vec<String>>>,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> bool {
        if stack.contains(node) {
            return true;
        }

        if visited.contains(node) {
            return false;
        }

        visited.insert(node.clone());
        stack.insert(node.clone());

        if let Some(Some(neighbors)) = graph.get(node) {
            for neighbor in neighbors {
                if self.is_cyclic(neighbor, graph, visited, stack) {
                    return true;
                }
            }
        }

        stack.remove(node);
        false
    }
}
