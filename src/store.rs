use crate::common::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub bot_token: String,
    pub role_to_add: u64,
    pub role_prefix_to_remove: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContainedUser {
    pub user_id: u64,
    pub name: String,
    pub role_ids_to_restore: Vec<u64>,
}

impl ContainedUser {
    pub fn new<S: Into<String>>(user_id: &u64, name: S, role_ids_to_restore: &[u64]) -> Self {
        Self {
            user_id: *user_id,
            name: name.into(),
            role_ids_to_restore: role_ids_to_restore.to_owned(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Status {
    pub to_restore: Vec<ContainedUser>,
}

pub fn load<D: DeserializeOwned>(path: &Path) -> Result<D> {
    let content = fs::read_to_string(path)?;
    let instance = serde_json::from_str(&content)?;
    Ok(instance)
}

pub fn save<S: Serialize>(instance: S, path: &Path) -> Result<()> {
    let content = serde_json::to_string(&instance)?;
    fs::write(path, &content)?;
    Ok(())
}

// types for Serenity

pub struct ConfigContainer;

impl TypeMapKey for ConfigContainer {
    type Value = Config;
}

pub struct StatusContainer;

impl TypeMapKey for StatusContainer {
    type Value = Status;
}
