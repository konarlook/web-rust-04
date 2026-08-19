use anyhow::{Context, Result};
use std::fs;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;

const TOKEN_FILE: &str = ".blog_token";

pub fn save(token: &str) -> anyhow::Result<()> {
    write_private(token).with_context(|| format!("не удалось записать {TOKEN_FILE}"))
}

pub fn load() -> Result<Option<String>> {
    match fs::read_to_string(path()) {
        Ok(content) => {
            let token = content.trim();
            Ok((!token.is_empty()).then(|| token.to_owned()))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("не удалось прочиать {TOKEN_FILE}")),
    }
}

fn path() -> PathBuf {
    PathBuf::from(TOKEN_FILE)
}

#[cfg(unix)]
fn write_private(token: &str) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path())?;

    file.write_all(token.as_bytes())?;
    fs::set_permissions(path(), fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn write_private(token: &str) -> std::io::Result<()> {
    fs::write(path(), token)
}
