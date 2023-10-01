use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfigJSON {
    #[serde(rename = "servers")]
    servers: Vec<ServerConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(rename = "username")]
    username: String,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "type")]
    server_type: String,

    #[serde(rename = "host")]
    host: String,

    #[serde(rename = "location")]
    location: String,

    #[serde(rename = "password")]
    password: String,
}

impl ServerConfigJSON {

    pub(crate) fn build_server_config_json() -> ServerConfigJSON {
        let file = File::open("config.json").expect("failed to open file");
        let reader = BufReader::new(file);
        let j:ServerConfigJSON = serde_json::from_reader(reader).expect("failed to read json");
        j
    }

    pub(crate) fn check_name_pass(&self, username:&str, password:&str) -> Option<usize> {
        for (index, s) in self.servers.iter().enumerate() {
            if s.username==username && s.password==password {
                return Some(index);
            }
        }
        return None;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfigWithoutPassword {
    #[serde(rename = "username")]
    username: String,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "type")]
    server_type: String,

    #[serde(rename = "host")]
    host: String,

    #[serde(rename = "location")]
    location: String,

    #[serde(skip_serializing)]
    password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfigJSONWithoutPassword {
    #[serde(rename = "servers")]
    servers: Vec<ServerConfigWithoutPassword>,
}

impl ServerConfigWithoutPassword {
    pub(crate) fn build_server_config_json() -> ServerConfigJSONWithoutPassword {
        let file = File::open("config.json").expect("failed to open file");
        let reader = BufReader::new(file);
        let j:ServerConfigJSONWithoutPassword = serde_json::from_reader(reader).expect("failed to read json");
        j
    }
}