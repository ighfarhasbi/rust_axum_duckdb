use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mahasiswa {
    pub id: i32,
    pub nama: String,
    pub tgl_lahir: String,
}

