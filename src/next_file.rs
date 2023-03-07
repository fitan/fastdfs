use std::fs::File;
use std::io::{Read, Seek, Write};
use anyhow::Context;
use tokio::fs;
use tokio::io::AsyncSeekExt;
use tokio::sync::Mutex;

pub struct NextFile {
    path: String,
    // 游标记录点
    cursor:  u64,
    // 插入值文件句柄
    inset_file:    File,
    // 读取值文件句柄
    read_file:   File,
}

impl NextFile {
    pub fn new(file_name: String) -> anyhow::Result<Mutex<NextFile>> {
        let inset_file = std::fs::OpenOptions::new().create(true).append(true).open(&file_name).context("open")?;
        let read_file = std::fs::OpenOptions::new().read(true).open(&file_name).context("open")?;
        Ok(Mutex::new(NextFile {
            path: file_name,
            cursor: 0,
            inset_file,
            read_file,
        }))
    }



    // 插入value
    pub async fn inset(&mut self, value: &String) -> anyhow::Result<()> {
        let data_byte = value.as_bytes();
        let data_len = format!("{:x}",data_byte.len() as u64);

        self.inset_file.write_all(format!("{}{}", data_len, value).as_bytes()).context("write_all")
    }

    pub async fn next(&mut self) -> anyhow::Result<String> {
        let mut len_buffer = [0;16];
        self.read_file.read(&mut len_buffer).context("read")?;
        let len = u64::from_str_radix(std::str::from_utf8(&len_buffer).context("from_utf8")?, 16).context("from_str_radix")?;
        let mut data_buffer = vec![0;len as usize];
        self.read_file.read(&mut data_buffer).context("read")?;
        let data = std::str::from_utf8(&data_buffer).context("from_utf8")?;
        Ok(data.to_string())
    }
}