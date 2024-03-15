use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Config {
    pub hostname: String,
    pub http_scheme: String,
    pub frontend_url: String,
    pub database: DatabaseConfig,
    pub instance: InstanceConfig,
    pub upload_dir: String,
}

impl Config {
    pub fn base_url(&self) -> String {
        format!("{}://{}", self.http_scheme, self.hostname)
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InstanceConfig {
    pub name: String,
    pub description: String,
}

#[test]
fn read_config_yaml() {
    use std::io::Read;

    // read from lightpub.yml.sample
    let mut file = std::fs::File::open("lightpub.yml.sample").unwrap();
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
                host: "127.0.0.1".to_string(),
                port: 3306,
                name: "lightpub".to_string(),
                user: "root".to_string(),
                password: "lightpub".to_string(),
                max_connections: 5
            },
            instance: InstanceConfig {
                name: "Lightpub dev".to_string(),
                description: "Lightpub development server".to_string()
            },
            upload_dir: "uploads".to_string(),
        }
    )
}
