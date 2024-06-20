use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]

pub struct Claims {
    exp: usize,
    iat: usize,
}

