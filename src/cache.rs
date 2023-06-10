use std::collections::HashMap;

use bytes::Bytes;

/// ### 文件缓存 FileCache
/// 
/// 能够容纳一定量的文件的缓存，供Response使用
pub struct FileCache {
    cache: HashMap<String, Bytes>,
    capacity: usize,    // 最大容纳的缓存数
    size: usize,        // 当前缓存数
    first: String,
}

impl FileCache {
    /// 通过指定缓存大小来创建一个新的缓存
    pub fn from_capacity(capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
            capacity,
            size: 0,
            first: String::new(),
        }
    }

    /// 将一段数据放入缓存。
    /// 
    /// - 如果缓存已满，则替换掉最早进入缓存的数据
    /// - 如果缓存未满，则直接放入
    pub fn push(&mut self, filename: &str, bytes: Bytes) {
        let filename_str = filename.to_string();
        // 已达到最大容量，替换掉最旧的缓存记录
        if self.size == self.capacity {
            self.cache.remove(&self.first);
            self.first = filename_str.clone();
        } else {
            self.size += 1;
        }
        self.cache.insert(filename_str, bytes);
    }

    /// 在缓存中查找数据
    /// 
    /// 参数：
    /// - `filename`：文件名，也是缓存的key。
    /// 
    /// 返回：
    /// - `bool`：查找是否失败。失败为`true`，此时`Bytes`为空。
    /// - `Bytes`：找到的数据。仅当`bool`为`false`时此字段才有意义。
    pub fn find(&self, filename: &str) -> (bool, Bytes) {
        match self.cache.get(filename) {
            Some(b) => (false, b.clone()),
            None => (true, Bytes::new()),
        }
    }
}