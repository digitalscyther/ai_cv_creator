use std::fs;
use crate::user::User;

pub async fn load_user(id: u64) -> Option<User> {
    let path = format!("files/user_{}.json", id);
    if let Ok(contents) = fs::read_to_string(&path) {
        if let Ok(user) = serde_json::from_str::<User>(&contents) {
            return Some(user);
        }
    }
    None
}

pub async fn save_user(user: &User) -> Result<(), &'static str> {
    let path = format!("files/user_{}.json", user.id);
    let data = serde_json::to_string(&user).map_err(|_| "Serialization error")?;

    fs::create_dir_all("files").map_err(|_| "Failed to create directory")?;
    fs::write(&path, data).map_err(|_| "Failed to write user data")?;

    Ok(())
}