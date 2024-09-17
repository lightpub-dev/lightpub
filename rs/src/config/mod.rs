use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Config {
    pub hostname: String,
    pub http_scheme: String,
    pub frontend_url: String,
    pub database: DatabaseConfig,
    pub queue: QueueConfig,
    pub instance: InstanceConfig,
    pub upload_dir: String,
    pub dev: DevConfig,
    pub federation: FederationConfig,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct FederationConfig {
    pub enabled: bool,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct DevConfig {
    pub debug: bool,
    pub ssl_verify: bool,
}

impl Config {
    pub fn base_url(&self) -> String {
        format!("{}://{}", self.http_scheme, self.hostname)
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct QueueConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InstanceConfig {
    pub name: String,
    pub description: String,
    pub open_registration: bool,
}

#[test]
fn read_config_yaml() {
    use std::io::Read;

    // read from lightpub.yml.sample
    let mut file = std::fs::File::open("../lightpub.yml.sample").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // Deserialize the YAML to a Config instance
    let config: Config = serde_yaml::from_str(&contents).expect("Unable to deserialize YAML");

    assert_eq!(
        config,
        Config {
            hostname: "lightpub.tinax.local".to_string(),
            http_scheme: "https".to_string(),
            frontend_url: "http://localhost:5173/#".to_string(),
            database: DatabaseConfig {
                path: "lightpub:lightpub@127.0.0.1:3306/lightpub".to_string()
            },
            queue: QueueConfig {
                host: "127.0.0.1".to_string(),
                port: 5672,
                user: "guest".to_string(),
                password: "guest".to_string()
            },
            instance: InstanceConfig {
                name: "Lightpub dev".to_string(),
                description: "Lightpub development server".to_string(),
                open_registration: true,
            },
            upload_dir: "uploads".to_string(),
            dev: DevConfig {
                debug: true,
                ssl_verify: false
            },
            federation: FederationConfig { enabled: true }
        }
    )
}
