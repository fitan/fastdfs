use std::sync::Arc;
use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use crate::storage::Storage;

// 上传文件
pub async fn file_upload(mut multipart: Multipart, State(storage): State<Arc<Storage>>) -> String {
    let s = storage.clone().save_file(multipart).await;
    if s.is_err() {
        return s.err().unwrap().to_string();
    }
    s.unwrap()
}


// 下载文件
pub async fn file_get(Path(name): Path<String>, State(storage): State<Arc<Storage>>) -> (HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    let mut body = Vec::new();
    let mut file = storage.get_file(&name).await;
    if file.is_err() {
        headers.insert("status", "404".parse().unwrap());
        body = "file not found".as_bytes().to_vec();
        return (headers, body);
    }
    headers.insert("status", "200".parse().unwrap());
    (headers, file.unwrap())
}