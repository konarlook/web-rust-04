pub struct User {
    id: u64,
    username: String,
    email: String,
    password_hash: String,
    created_at: chrono::NaiveDateTime,
}

pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub struct AuthRequest {
    pub username: String,
    pub password: String,
}
