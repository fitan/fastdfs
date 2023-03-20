use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use crate::storage::Storage;

// 上传文件
pub async fn file_upload(State(storage): State<Arc<Storage>>, mut multipart: Multipart) -> String {
    let s = storage.clone().save_file(multipart).await;
    if s.is_err() {
        return s.err().unwrap().to_string();
    }
    s.unwrap()
}


// 下载文件
pub async fn file_get(Query(params): Query<HashMap<String, String>>, State(storage): State<Arc<Storage>>) -> (HeaderMap, Vec<u8>) {
    // let file_name = path.strip_prefix("/file/").unwrap_or(&path).to_string();
    let name = params.get("name").unwrap().to_string();
    tracing::info!("file_name: {}", &name);
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