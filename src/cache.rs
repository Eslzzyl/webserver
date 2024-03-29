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
        if capacity == 0 {
            panic!("调用from_capacity时指定的大小是0。如果需要自动设置大小，请在调用处进行处理，而不是传入0");
        }
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
    /// ## 参数：
    /// - `filename`：文件名，也是缓存的key。
    pub fn find(&self, filename: &str) -> Option<&Bytes> {
        self.cache.get(filename)
    }
}