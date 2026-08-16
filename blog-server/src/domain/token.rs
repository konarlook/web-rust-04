use crate::domain::error::UserError;

pub trait TokenIssuer: Send + Sync {
    fn issue(&self, user_id: i64, username: &str) -> Result<String, UserError>;
}
