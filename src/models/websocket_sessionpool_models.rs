use dashmap::DashMap;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::{Arc, LazyLock, Mutex};

pub enum CleanupType {
    Full,
    Search { key: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {
    pub last_active: u64,
    pub value: String,
}

// For the heap, we need to store the key as well, so we can find/remove from the pool.
#[derive(Eq, PartialEq, Debug)]
pub struct ActivityTracker {
    pub last_active: u64,
    pub key: String,
}

type ConnectionPool = LazyLock<Arc<DashMap<String, Status>>>;

#[derive(Debug)]
pub struct ConnectionManager {
    pub connection_size: u32,
    pub pool: ConnectionPool,
    pub activity_tracker: Mutex<BinaryHeap<Reverse<ActivityTracker>>>, // Min-heap by last_active
}
