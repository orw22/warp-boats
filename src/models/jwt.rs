use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,   // expires at
    pub iat: usize,   // issued at
    pub sub: String,  // subject
    pub role: String, // user role
}
