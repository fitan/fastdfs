use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::Mutex;
use anyhow::Context;

struct NextFile {
    // 游标记录点
    cursor:  u64,
    // 插入值文件句柄
    inset_file:    Mutex<File>,
    // 读取值文件句柄
    read_file:   Mutex<File>,
}

impl NextFile {
    pub fn new(file_name: String) -> anyhow::Result<NextFile> {
        let inset_file = std::fs::OpenOptions::new().create(true).append(true).open(&file_name).context("open")?;
        let read_file = std::fs::OpenOptions::new().read(true).open(&file_name).context("open")?;
        Ok(NextFile {
            cursor: 0,
            inset_file: Mutex::new(inset_file),
            read_file: Mutex::new(read_file),
        })
    }

    // 插入value
    pub fn inset(&mut self, value: &String) -> anyhow::Result<()> {
        let mut file = self.inset_file.lock().context("lock")?;
        let data_byte = value.as_bytes();
        let data_len = format!("{:x}",data_byte.len() as u64);

        file.write_all(format!("{}{}", data_len, value).as_bytes()).context("write_all")
    }

    pub fn next(&mut self) -> anyhow::Result<String> {
        let mut file = self.read_file.lock().context("lock")?;
        let mut len_buffer = [0;16];
        file.read(&mut len_buffer).context("read")?;
        let len = u64::from_str_radix(std::str::from_utf8(&len_buffer).context("from_utf8")?, 16).context("from_str_radix")?;
        let mut data_buffer = vec![0;len as usize];
        file.read(&mut data_buffer).context("read")?;
        let data = std::str::from_utf8(&data_buffer).context("from_utf8")?;
        Ok(data.to_string())
    }
}