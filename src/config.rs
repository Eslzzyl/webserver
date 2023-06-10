use serde_derive::Deserialize;
use serde_derive::Serialize;

use std::fs::File;
use std::io::prelude::*;
use core::str;
use log::error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    www_root: String,
    port: u16,
}

impl Config {
    pub fn new() -> Self {
        Self {
            www_root: ".".to_string(),
            port: 7878,
        }
    }

    pub fn from_toml(filename: &str) -> Self {
        // 打开文件
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => panic!("no such file {} exception:{}", filename, e)
        };
        // 读文件到str
        let mut str_val = String::new();
        match file.read_to_string(&mut str_val) {
            Ok(s) => s,
            Err(e) => panic!("Error Reading file: {}", e)
        };

        // 尝试读配置文件，若成功则返回，若失败则返回默认值
        match toml::from_str(&str_val) {
            Ok(t) => t,
            Err(_) => {
                error!("无法成功从配置文件构建配置对象，使用默认配置");
                Config::new()
            }
        }
    }
}

impl Config {
    pub fn www_root(&self) -> &str {
        &self.www_root
    }
    
    pub fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing_read() {
        let config = Config::from_toml("files/config.toml");
        assert_eq!(config.www_root(), ".");
        assert_eq!(config.port(), 7878);
    }
}