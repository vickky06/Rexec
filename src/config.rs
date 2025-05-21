use serde::Deserialize;
use std::fs;
use once_cell::sync::OnceCell;

pub static GLOBAL_CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug, Deserialize)]
pub struct Dockerfiles {
    pub python: String,
    pub javascript: String,
    pub java: String,
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub tar_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Constants {
   pub dockerfile : String,
}

#[derive(Debug, Deserialize)]
pub struct Build{
    pub service_port: i32,
    pub service_name: String,
}


#[derive(Debug, Deserialize)]
pub struct Config {
    pub dockerfiles: Dockerfiles,
    pub paths: Paths,
    pub constants: Constants,
    pub build: Build,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let content = fs::read_to_string(path).expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse config file")
    }
}