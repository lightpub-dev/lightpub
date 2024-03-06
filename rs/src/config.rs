use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Config {
    pub hostname: String,
    pub http_scheme: String,
    pub frontend_url: String,
    pub database: DatabaseConfig,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
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
            }
        }
    )
}
