use core::slice::SlicePattern;
use std::collections::HashMap;
use std::io::{Bytes, Write};
use std::path::Path;
use std::str::pattern::Pattern;
use anyhow::Context;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::Multipart;
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use crc32fast::Hasher;
use hex::ToHex;
use tokio::io::split;
use tokio::sync::Mutex;
use crate::wrr::WeightedRoundRobin;


pub struct Storage {
    // 本地ip
    local_ip: String,
    // 组名
    group_name: String,

    // 目录数量
    dir_count: u8,
    // 根目录列表
    root_dirs: Vec<RootDir>,

    root_dirs_map: HashMap<String, RootDir>,

    // 根目录磁盘大小
    // 临时目录
    tmp_dir: String,

    // 权重获取根目录
    rrw_root_dirs: Mutex<WeightedRoundRobin>,

    // hasher
    hasher: Mutex<Hasher>,

}

struct RootDir {
    // 名字
    name: String,
    // 根目录
    dir: String,
    // 读写权限
    read_write: bool,
}

impl Storage {
    pub fn new(dir: String) -> Storage {
        Storage {
            level: 2,
            root_dirs: vec![],
            tmp_dir: "/tmp".to_string(),
            rrw_root_dirs: ,
        }
    }

    pub async fn get_file(&self, name: &String) -> anyhow::Result<&[u8]> {
        let real_file = self.decode_file_name_to_real_file_name(name)?;
        let
        tokio::fs::read(&real_file).ret
    }

    pub fn decode_file_name_to_real_file_name(&self, name: &String) -> anyhow::Result<String> {
        let (_, dir_name, sub_dir, sud_idr, id, file_ext_name) = decode_file_name(name)?;
        let root_path = self.root_dirs_map.get(&dir_name).context("not found root_path")?;
        Ok(format!("{}/{}/{}/{}.{}", root_path.dir, sub_dir, sud_idr, id, file_ext_name))
    }


    // 保存文件
    pub async fn save_file(&self, mut payload: Multipart) -> anyhow::Result<String> {
        let mut f = std::fs::OpenOptions::new().write(true).create(true).open(&self.tmp_dir).context("open")?;

        if let Some(field) = payload.next_field().await? {
            let content_type = field.content_type().context("not found content_type")?.to_string();
            let file_name = field.file_name().context("not found file_name")?.to_string();
            let suffix_name = Path::new(&file_name).extension().context("not found suffix_name")?.to_string_lossy().to_string();
            let data = field.bytes().await?;
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            let id = gen_file_id(&self.local_ip, timestamp.to_owned(), data.len() as u64, gen_file_crc32(&data)?).context("gen file id")?;
            let root_dir = self.get_root_dir()?;
            let crc = gen_file_crc32(&data)?;
            let sub_dir = inset_dir_by_key(crc)?;
            let new_file_name = gen_file_name(&self.group_name, &root_dir.name, &sub_dir.to_string(), &sub_dir.to_string(), &id, &suffix_name)?;
            let real_file_name = format!("{}/{}/{}/{}.{}", &root_dir.dir, &sub_dir, &sub_dir, &id, &suffix_name);


            let tmp_file = format!("{}/{}", &self.tmp_dir, &id);
            tokio::fs::write(&tmp_file, data).await?;

            tokio::fs::rename(tmp_file, real_file_name).await?;

            return Ok(new_file_name)
        }

        Err(anyhow::anyhow!("not found file"))
    }

    // 轮训获取根目录
    pub fn get_root_dir(&self) -> anyhow::Result<RootDir> {
        let mut index = 0;
        let mut root_dir: Option<RootDir> = None;
        for i in 0..self.root_dirs.len() {
            if self.root_dirs[i].read_write {
                index = i;
                root_dir = Some(self.root_dirs[i]);
                break;
            }
        }
        return if let Some(v) = root_dir {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("no root dir"))
        }
    }




    pub fn upload_file(&self, name: &String) -> anyhow::Result<String> {
        let dir = inset_dir_by_key(name)?;
        std::fs::rename(format!("{}/{}", self.tmp_dir, name), &dir).context("rename")?;
        Ok(format!("{}/{}", dir, name))
    }
    // pjw hash


    // 根据key 时间戳 文件目录 生成文件名
    pub fn file_by_key(&self, key: &String, timestamp: u64) -> anyhow::Result<String> {
        let dir = inset_dir_by_key(key)?;
        let file = format!("{}/{}_{}", dir, timestamp, key);
        Ok(file)
    }
}


// pjw hash
fn inset_dir_by_key(crc: u32) -> anyhow::Result<u32> {
    Ok(crc % 255)
}

fn gen_file_crc32(data: &[u8]) -> anyhow::Result<u32> {
    Ok(crc32fast::hash(data))
}

// 生成文件id: ip地址_文件创建时间戳_文件大小_文件crc32_随机数
fn gen_file_id(ip: &String, timestamp: u64, size: u64, crc32: u32) -> anyhow::Result<String> {
    let rand = rand::random::<u32>();
    let file_id = format!("{}_{}_{}_{}_{}", ip, timestamp, size, crc32, rand);
    Ok(base64::encode(&file_id))
}

fn decode_file_id(s: &String) -> anyhow::Result<(String,u64, u64, u32, u32)> {
    let d = base64::decode(s).context("decode base64")?;
    let s = String::from_utf8(d).context("from utf8")?;
    let v: Vec<&str> = s.split("_").collect();
    if v.len() != 5 {
        return Err(anyhow::anyhow!("invalid file name"));
    }
    let ip = v[0].to_string();
    let timestamp = v[1].parse::<u64>().context("parse timestamp")?;
    let size = v[2].parse::<u64>().context("parse size")?;
    let crc32 = v[3].parse::<u32>().context("parse crc32")?;
    let rand = v[4].parse::<u32>().context("parse rand")?;
    Ok((ip,timestamp, size, crc32, rand))
}

// 文件名生成:  组名_存储目录名字_子目录名字0_自目录名字1_文件id_文件后缀
fn gen_file_name(group_name: &String, dir_name: &String, sub_dir_name0: &String, sub_dir_name1: &String, file_id: &String, file_ext_name: &String) -> anyhow::Result<String> {
    let file_name = format!("{}/{}/{}/{}/{}/{}", group_name, dir_name, sub_dir_name0, sub_dir_name1, file_id, file_ext_name);
    Ok(file_name)
}

// 解析文件名
fn decode_file_name(file_name: &String) -> anyhow::Result<(String, String, String, String, String, String)> {
    let v: Vec<&str> = file_name.split("/").collect();
    if v.len() != 7 {
        return Err(anyhow::anyhow!("invalid file name"));
    }
    let group_name = v[0].to_string();
    let dir_name = v[1].to_string();
    let sub_dir_name0 = v[2].to_string();
    let sub_dir_name1 = v[3].to_string();
    let file_id = v[4].to_string();
    let file_ext_name = v[5].to_string();
    Ok((group_name, dir_name, sub_dir_name0, sub_dir_name1, file_id, file_ext_name))
}
