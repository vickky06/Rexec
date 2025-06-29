use crate::{
    models::config_models::Config,
    services::{
        all_session_services::{
            session_cache_service::SessionCache,
            session_management_service::SessionManagementService,
        },
        websocket::websocket_sessionpool_service::ConnectionManager,
    },
};
use once_cell::sync::OnceCell;
use std::fs;
use tokio::sync::Mutex;

pub static GLOBAL_CONFIG: OnceCell<Mutex<Config>> = OnceCell::new();

pub const CONFIG_FILE: &str = "config.toml";

impl Config {
    pub fn new() -> Self {
        let path = CONFIG_FILE;
        let content = fs::read_to_string(path).expect("Failed to read config file");
        let config: Config = toml::from_str(&content).expect("Failed to parse config file");
        config
    }

    pub fn init(&mut self) {
        let session_mangement_service = SessionManagementService::new();
        self.session_management_service = Some(session_mangement_service);
        let session_cache: &'static SessionCache = SessionCache::new();
        self.session_cache_service = Some(session_cache);
        let websocket_connection_manager = ConnectionManager::get_connection_manager();
        self.websocket_seesion_pool = Some(websocket_connection_manager)
    }

    pub fn set_session_management_service(&mut self, sms: &'static SessionManagementService) {
        self.session_management_service = Some(sms);
    }
    pub fn set_websocket_connection_manager(&mut self, wbcm: &'static ConnectionManager) {
        self.websocket_seesion_pool = Some(wbcm)
    }
    pub fn set_session_cache(&mut self, session_cache: &'static SessionCache) {
        self.session_cache_service = Some(session_cache)
    }
}

pub async fn get_global_config<F, R>(f: F) -> R
where
    F: FnOnce(&Config) -> R,
{
    let guard = GLOBAL_CONFIG
        .get()
        .expect("Global config not initialized")
        .lock()
        .await;
    f(&*guard)
}

pub async fn get_global_config_mut() -> tokio::sync::MutexGuard<'static, Config> {
    GLOBAL_CONFIG
        .get()
        .expect("Global config mutex not initialized")
        .lock()
        .await
}
pub fn set_global_config(config: Config) {
    GLOBAL_CONFIG
        .set(Mutex::new(config))
        .expect("Failed to set global config");
}
