use serde::Serialize;

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub message: String,
    pub status: String,
}
