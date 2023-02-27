use anyhow::Context;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};


struct Storage {
    // 层级
    level: u8,
    // 目录数量
    dir_count: u8,
    // 根目录列表
    root_dirs: Vec<RootDir>,

    // 轮询目录索引
    root_dir_index: u8,
    // 根目录磁盘大小
    // 临时目录
    tmp_dir: String,

}

struct RootDir {
    // 根目录
    root_dir: String,
    // 读写权限
    read_write: bool,
}

impl Storage {
    pub fn new(dir: String) -> Storage {
        Storage {
            level: 2,
            dir_count: 16,
            root_dirs: vec![],
            tmp_dir: "/tmp".to_string(),
        }
    }

    // 轮训获取根目录
    pub fn get_root_dir(&self) -> anyhow::Result<String> {
        let mut index = 0;
        let mut root_dir = String::new();
        for i in 0..self.root_dirs.len() {
            if self.root_dirs[i].read_write {
                index = i;
                root_dir = self.root_dirs[i].root_dir.clone();
                break;
            }
        }
        if root_dir.is_empty() {
            return Err(anyhow::anyhow!("no root dir"));
        }
        Ok(root_dir)
    }




    pub fn upload_file(&self, name: &String) -> anyhow::Result<String> {
        let dir = self.dir_by_key(name)?;
        std::fs::rename(format!("{}/{}", self.tmp_dir, name), &dir).context("rename")?;
        Ok(format!("{}/{}", dir, name))
    }
    // pjw hash
    pub fn dir_by_key(&self, key: &String) -> anyhow::Result<String> {
        let mut hash = 0;
        for c in key.chars() {
            hash = (hash << 4) +  c as u32;
            let g = hash & 0xf0000000;
            if g != 0 {
                hash ^= g >> 24;
            }
            hash &= !g;
        }
        let mut dir = String::new();
        for i in 0..self.level {
            let index = (hash >> (i * 4)) & 0xf;
            dir = format!("{}/{}", dir, index);
        }
        Ok(dir)
    }

    // 根据key 时间戳 文件目录 生成文件名
    pub fn file_by_key(&self, key: &String, timestamp: u64) -> anyhow::Result<String> {
        let dir = self.dir_by_key(key)?;
        let file = format!("{}/{}_{}", dir, timestamp, key);
        Ok(file)
    }
}


// 文件名生成  根目录_时间戳_文件大小_随机数
fn gen_file_name(root_dir: String, size: u64) -> anyhow::Result<String> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).context("SystemTime before UNIX EPOCH!")?.as_secs();
    let rand = rand::random::<u32>();
    let file_name = format!("{}_{}_{}_{}", root_dir,timestamp, size, rand);
    Ok(base64::encode(&file_name))
}

fn decode_file_name(s: &String) -> anyhow::Result<(String,u64, u64, u32)> {
    let d = base64::decode(s).context("decode base64")?;
    let s = String::from_utf8(d).context("from utf8")?;
    let v: Vec<&str> = s.split("_").collect();
    if v.len() != 4 {
        return Err(anyhow::anyhow!("invalid file name"));
    }
    let root_dir = v[0].to_string();
    let timestamp = v[1].parse::<u64>().context("parse timestamp")?;
    let size = v[2].parse::<u64>().context("parse size")?;
    let rand = v[3].parse::<u32>().context("parse rand")?;
    Ok((root_dir,timestamp, size, rand))
}

