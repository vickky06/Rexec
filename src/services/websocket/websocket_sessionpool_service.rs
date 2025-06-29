use crate::{
    models::websocket_sessionpool_models::{ActivityTracker, CleanupType, Status},
    services::helper_services::config_service::get_global_config,
    utils::helper_utils::get_current_timestamp,
};

pub use crate::models::websocket_sessionpool_models::ConnectionManager;

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    sync::{Arc, LazyLock, Mutex},
};

static WEBSOCKET_SESSION_POOL: OnceCell<ConnectionManager> = OnceCell::new();

impl Ord for ActivityTracker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.last_active.cmp(&other.last_active)
    }
}
impl PartialOrd for ActivityTracker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ActivityTracker {
    pub fn new(last_active: u64, key: String) -> Self {
        ActivityTracker { last_active, key }
    }
}

impl ConnectionManager {
    fn new() -> &'static ConnectionManager {
        if WEBSOCKET_SESSION_POOL.get().is_none() {
            WEBSOCKET_SESSION_POOL
                .set(ConnectionManager {
                    pool: LazyLock::new(|| Arc::new(DashMap::new())),
                    connection_size: 0,
                    activity_tracker: Mutex::new(BinaryHeap::new()),
                })
                .ok();
        }
        WEBSOCKET_SESSION_POOL.get().unwrap()
    }

    pub fn get_connection_manager() -> &'static ConnectionManager {
        Self::new()
    }

    pub fn add_connection(&self, key: &str, value: String) {
        let status = self.pool.insert(key.to_string(), Status::new(value));
        let activity_record = ActivityTracker::new(status.unwrap().last_active, key.to_string());
        self.activity_tracker
            .lock()
            .unwrap()
            .push(Reverse(activity_record));
        self.cleanup(CleanupType::Full);
    }

    pub fn get_connection(self, key: &str) -> Option<Status> {
        self.pool.get(key).map(|status| status.clone())
    }

    pub fn remove_connection(&self, key: &str) {
        self.pool.remove(key);
        self.cleanup(CleanupType::Search {
            key: key.to_string(),
        });
    }

    async fn cleanup(&self, cleanup_type: CleanupType) {
        match cleanup_type {
            CleanupType::Full => {
                // Assuming get_global_configs().WebSocketPoolConfig.max_connections exists and returns usize
                let max_connections = get_global_config(|config| config.clone())
                    .await
                    .websocket_pool_config
                    .max_connections;
                let mut activity_tracker = self.activity_tracker.lock().unwrap();
                while activity_tracker.len() > max_connections {
                    if let Some(Reverse(activity)) = activity_tracker.pop() {
                        self.pool.remove(&activity.key);
                    }
                }
            }
            CleanupType::Search { key } => {
                // Remove the entry with the given key from activity_tracker
                let mut activity_tracker = self.activity_tracker.lock().unwrap();
                let mut temp = BinaryHeap::new();
                while let Some(Reverse(activity)) = activity_tracker.pop() {
                    if activity.key != key {
                        temp.push(Reverse(activity));
                    }
                }
                *activity_tracker = temp;
            }
        }
    }
}

impl Status {
    pub fn new(value: String) -> Self {
        Status {
            last_active: get_current_timestamp(),
            value,
        }
    }

    pub fn was_last_active(&self) -> u64 {
        self.last_active
    }

    pub fn set_last_active(&mut self) {
        self.last_active = get_current_timestamp();
    }

    pub fn upsert_value(&mut self, value: String) {
        self.value = value;
        self.set_last_active();
    }
}
