use std::sync::Arc;
use axum::extract::{Multipart, Path, State};
use crate::storage::Storage;

// 上传文件
pub fn file_upload(Path(name): Path<String>,Multipart { inner }: Multipart, State(storage): State<Arc<Storage>>) -> anyhow::Result<()> {
    storage.clone().save_file()

}