use serde::Serialize;

#[derive(Serialize)] // tampilkan ke response
pub struct ResponseModel<T> {
    pub kode: i32,
    pub status: String,
    pub data: Option<T>
}

#[derive(Serialize, Debug, Clone)]
pub struct ResponseUser  {
    pub id: i32,
    pub username: String,
    // pub token: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ResponseLoginUser  {
    pub id: i32,
    pub username: String,
    pub token: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ResponseLoginRefUser  {
    pub id: i32,
    pub username: String,
    pub token: String,
    pub refresh_token: String,
}