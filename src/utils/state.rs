use crate::utils::{Client, Config};

pub struct AppState {
    pub client: Client,
    pub config: Config,
}