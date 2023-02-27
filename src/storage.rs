struct Storage {
    // 层级
    level: u8,
    // 目录数量
    dir_count: u8,
    // 根目录列表
    root_dirs: Vec<RootDir>,
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
    pub fn upload_file(&self, name: &String) -> anyhow::Result<String> {
        let dir = self.dir_by_key(name)?;
        std::fs::rename(format!("{}/{}", self.tmp_dir, name), &dir).context("rename")?;
        Ok(format!("{}/{}", dir, name))
    }

    // hash文件名字分散在255个目录中
    fn dir_by_key(&self, key: &String) -> anyhow::Result<String> {
        let hash = hex::encode(Md5::digest(key.as_bytes()));
        let mut dir = String::new();
        for i in 0..self.level {
            let index = hash.chars().nth(i as usize).unwrap().to_digit(16).unwrap() as u8;
            dir = format!("{}/{}", dir, index);
        }
        Ok(dir)
    }
}