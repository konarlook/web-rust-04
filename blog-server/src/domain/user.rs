use crate::domain::error::UserRegisterRequestError;

pub struct User {
    id: u64,
    username: String,
    email: String,
    password_hash: String,
    created_at: chrono::NaiveDateTime,
}

pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}
