use serde::Serialize;

#[derive(Serialize)] // tampilkan ke response
pub struct ResponseModel<T> {
    pub kode: i32,
    pub status: String,
    pub data: Option<T>
}

#[derive(Serialize, Debug)]
pub struct ResponseUser  {
    pub id: i32,
    pub username: String,
    pub token: String,
}