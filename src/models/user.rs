use crate::schema::users;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

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
