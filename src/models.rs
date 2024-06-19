use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mahasiswa {
    pub id: i32,
    pub nama: String,
    pub tgl_lahir: String,
    pub user_id: i32
}

#[derive(Deserialize)]
pub struct CreateMahasiswa {
    pub id: i32,
    pub nama: String,
    pub tgl_lahir: String,
}

#[derive(Deserialize)]
pub struct ReqUser {
    pub id: i32,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ReqLogin {
    pub username: String,
    pub password: String,
}