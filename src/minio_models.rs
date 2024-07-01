use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileUploadResponse {
    file_id: String,
    file_name: String,
    created_date: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteResponse {
    message: String,
}