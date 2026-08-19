use web_sys::Storage;

const TOKEN_KEY: &str = "blog_token";

fn storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn save_token(token: &str) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(TOKEN_KEY, token);
    }
}

pub fn load_token() -> Option<String> {
    storage()?.get_item(TOKEN_KEY).ok()?
}

pub fn clear_token() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(TOKEN_KEY);
    }
}
