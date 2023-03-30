use std::io::Cursor;
use anyhow::Context;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

enum Operation {
    Create,
    Update,
    Delete,
}

struct BindLog {
    path: String
}

impl BindLog {
    pub fn new(path: String) -> BindLog {
        BindLog{
            path
        }
    }

    // 在文件后追加数据， 内容由时间戳 + 文件路径 + 操作类型
    pub async fn inset(&mut self, timestamp: i64, path: &str, op: &str) -> anyhow::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path).await.context("open")?;
        let mut buffer = [0u8; 75];
        let mut cursor = Cursor::new(&mut buffer[..]);
        cursor.write_all(&timestamp.to_be_bytes()).await.context("write timestamp")?;
        cursor.write_all(path.as_bytes()).await.context("write path")?;
        cursor.write_all(op.as_bytes()).await.context("write op")?;
        file.write_all(&buffer).await.context("write_all")?;
        Ok(())
    }

    // 根据时间戳用二分法查找文件
    pub async fn get(&self, timestamp: i64) -> anyhow::Result<()> {
        let mut file_size = 0;
        match fs::metadata(&self.path).await.context("metadata") {
            Ok(meta) => {
                file_size = meta.len();
            },
            Err(e) => {
                return Err(e);
            }
        }
        let file = OpenOptions::new().read(true).open(&self.path).await.context("open")?;
        let current_offset = 0;
        Ok(())
    }
}