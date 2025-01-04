use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
