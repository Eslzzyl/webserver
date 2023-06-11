use serde_derive::Deserialize;
use serde_derive::Serialize;
use num_cpus;

use std::fs::File;
use std::io::prelude::*;
use core::str;
use log::{error, warn};

/// Config
/// 
/// 储存服务器需要的配置信息
/// 
/// - `www_root`: 服务器的Web根路径
/// - `port`: 要绑定的本机端口
/// - `worker_threads`: Tokio的工作线程数量。设置为`0`以使程序自动确定工作线程数量。默认值为CPU的核心数。
/// - `cache_size`: 文件缓存的大小，即能够容纳多少个文件。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    www_root: String,
    port: u16,
    worker_threads: usize,
    cache_size: usize,
}

impl Config {
    pub fn new() -> Self {
        Self {
            www_root: ".".to_string(),
            port: 7878,
            worker_threads: 0,
            cache_size: 5,
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
        let mut raw_config = match toml::from_str(&str_val) {
            Ok(t) => t,
            Err(_) => {
                error!("无法成功从配置文件构建配置对象，使用默认配置");
                Config::new()
            }
        };
        // config要求自动确定worker threads数量，使用当前cpu核心数量
        if raw_config.worker_threads == 0 {
            raw_config.worker_threads = num_cpus::get();
        }
        if raw_config.cache_size == 0 {
            warn!("cache_size被设置为0，但目前尚不支持禁用缓存，因此该值将被改为5。");
            raw_config.cache_size = 5;
        }
        raw_config
    }
}

impl Config {
    pub fn www_root(&self) -> &str {
        &self.www_root
    }
    
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn worker_threads(&self) -> usize {
        self.worker_threads
    }

    pub fn cache_size(&self) -> usize {
        self.cache_size
    }
}
