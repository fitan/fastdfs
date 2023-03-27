use std::collections::HashMap;
use std::sync::{Arc};
use axum::extract::{Multipart, Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use axum::response::Response;
use serde_json::de::Read;
use tokio::sync::Mutex;
use crate::storage::Storage;

// 上传文件
pub async fn file_upload(State(storage): State<Arc<Mutex<Storage>>>, mut multipart: Multipart) -> String {
    let s = storage.clone().lock().await.save_file(multipart).await;
    if s.is_err() {
        return s.err().unwrap().to_string();
    }
    s.unwrap()
}


// 下载文件
pub async fn file_get(Query(params): Query<HashMap<String, String>>, State(storage): State<Arc<Mutex<Storage>>>) -> (HeaderMap, Vec<u8>) {
    // let file_name = path.strip_prefix("/file/").unwrap_or(&path).to_string();
    let name = params.get("name").unwrap().to_string();
    tracing::info!("file_name: {}", &name);
    let mut headers = HeaderMap::new();
    let mut body = Vec::new();
    let mut file = storage.clone().lock().await.get_file(&name).await;
    if file.is_err() {
        headers.insert("status", "404".parse().unwrap());
        body = "file not found".as_bytes().to_vec();
        return (headers, body);
    }
    headers.insert("status", "200".parse().unwrap());
    (headers, file.unwrap())
}

#[derive(Debug,serde::Deserialize,serde::Serialize)]
pub struct RootDirSizeResponse {
    root_name: String,
    size: u64,
}

pub async fn root_dir_size(State(storage): State<Arc<Mutex<Storage>>>) -> Json<Vec<RootDirSizeResponse>> {
    let mut root_dir_size_response = Vec::new();

    for root_dir in &storage.clone().lock().await.root_dirs {
        root_dir_size_response.append(&mut vec![RootDirSizeResponse {
            root_name: root_dir.name.clone(),
            size: root_dir.next_file.get_cursor().await,
        }]);
    }
    Json(root_dir_size_response)
}