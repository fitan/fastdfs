use std::fs;
use std::fs::File;
use std::io::{Read, Cursor, Seek, Write, SeekFrom};
use std::sync::Mutex;
use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

pub struct SumSizeFile {
    path: String,
    // 游标记录点
    sum:  u64,
    // 插入值文件句柄
    inset_file: File,
    // 锁
    lock: Mutex<()>,
}

impl SumSizeFile {
    pub fn new(file_name: String) -> anyhow::Result<SumSizeFile> {
        let inset_file = std::fs::OpenOptions::new().create(true).append(true).open(&file_name).context("open")?;
        let mut sum = SumSizeFile {
            path: file_name,
            sum: 0,
            inset_file,
            lock: Mutex::new(()),
        };

        sum.sum();
        Ok(sum)
    }

    // 插入value
    pub async fn inset(&mut self, value: u64) -> anyhow::Result<()> {
        let _ = self.lock.lock();
        println!("inset value: {}, {:?}", value, value.to_be_bytes());
        self.inset_file.write_all(&value.to_be_bytes()).context("write_all")?;

        // let mut buffer = [0u8; 8];
        // let mut cursor = Cursor::new(&mut buffer[..]);
        // cursor.write_u64(value).await.context("write_u64")?;
        // let _ = self.lock.lock().await;
        //
        // self.inset_file.write(&buffer).context("write")?;
        self.sum = self.sum + value;
        tracing::info!("sum: {}", &self.sum);
        Ok(())
    }

    pub async fn get_cursor(&self) -> u64 {
        let _ = self.lock.lock();
        tracing::info!("get_cursor: {}", &self.sum);
        self.sum
    }

    // 根据游标取数据
    // pub async fn get(&mut self, start: u64) -> anyhow::Result<String> {
    //     let mut buffer = [0u8; 8];
    //     let mut file = fs::OpenOptions::new().read(true).open(&self.path).await.context("open")?;
    //     file.seek(SeekFrom::Start(start)).await.context("seek")?;
    //     file.read_exact(&mut buffer).await.context("read_exact")?;
    //
    //     let mut data_buffer = vec![0; u64::from_le_bytes(buffer) as usize];
    //     file.read_exact(&mut data_buffer).await.context("read_exact")?;
    //
    //     Ok(String::from_utf8(data_buffer)?)
    // }

    pub fn sum(&mut self) -> anyhow::Result<()> {
        let _ = self.lock.lock();
        let mut file = fs::OpenOptions::new().read(true).open(&self.path).context("open")?;
        let mut buffer = [0u8; 8];
        let mut sum = 0;
        loop {
            let read_len = file.read(&mut buffer).context("read")?;
            if read_len == 0 {
                break;
            }
            sum += u64::from_be_bytes(buffer);
        }
        self.sum = sum;
        let mut write_file = fs::OpenOptions::new().write(true).truncate(true).open(&self.path).context("open")?;
        write_file.write_all(&self.sum.to_be_bytes()).context("write_u64")?;
        write_file.sync_all().context("sync_all")?;
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use anyhow::Context;
    use crate::next_file::SumSizeFile;

    #[tokio::test]
    async fn it_works() {
        use super::*;
        let mut next_file = SumSizeFile::new("./next_file.txt".to_string()).unwrap();
        next_file.inset("100".len() as u64).await.unwrap();
        next_file.inset("100".len() as u64).await.unwrap();
        next_file.inset("100".len() as u64).await.unwrap();

        let mut file = File::open("./next_file.txt".to_string()).unwrap();
        let mut buffer = [0u8; 8];
        loop {
            match file.read_exact(&mut buffer) {
                Ok(_) => {
                    println!("read: {}", u64::from_be_bytes(buffer));
                }
                Err(e) => {
                    println!("read error: {}", e);
                    break;
                }
            }
        }

        next_file.sum().unwrap();
        let mut file = File::open("./next_file.txt".to_string()).unwrap();
        loop {
            match file.read_exact(&mut buffer) {
                Ok(_) => {
                    println!("sun read: {}", u64::from_be_bytes(buffer));
                }
                Err(e) => {
                    println!("sun read error: {}", e);
                    break;
                }
            }
        }

    }
}