use crate::schema::{boats, users};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

// Boat
#[derive(Deserialize, Serialize, Clone, Queryable, Selectable)]
#[diesel(table_name = boats)]
#[diesel(check_for_backend(Sqlite))]
pub struct Boat {
    pub id: i32,
    pub name: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub length: Option<f32>,
    pub beam: Option<f32>,
    pub is_available: i32,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = boats)]
pub struct NewBoat {
    pub name: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub length: Option<f32>,
    pub beam: Option<f32>,
    pub is_available: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateBoat {
    pub name: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub length: Option<f32>,
    pub beam: Option<f32>,
    pub is_available: Option<i32>,
}

// User
#[derive(Deserialize, Serialize, Clone, Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Sqlite))]
pub struct User {
    pub email: String,
    pub api_key: String,
    pub credit: i32,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub api_key: Option<String>,
}

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,   // expires at
    pub iat: usize,   // issued at
    pub sub: String,  // subject
    pub role: String, // user role
}
