pub struct Claims {
    user_id: u64,
    username: String,
    exp: chrono::NaiveDateTime,
}

pub struct JwtService {}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        return JwtService {};
    }

    pub fn generate_token(user_id: u64, username: String) {}

    pub fn verify_token(&self, token: &String) -> bool {
        return true;
    }
}
