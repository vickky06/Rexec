pub use crate::models::in_memory_session_cache_model::{Session, SessionCache};

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static SINGLETON_SESSION_MANAGEMENT_SERVICE: OnceCell<SessionCache> = OnceCell::new();

impl SessionCache {
    pub fn new() -> &'static Self {
        if !SINGLETON_SESSION_MANAGEMENT_SERVICE.get().is_some() {
            let in_memory_session = Self {
                sessions: Arc::new(DashMap::new()),
            };
            SINGLETON_SESSION_MANAGEMENT_SERVICE
                .set(in_memory_session)
                .ok();
        }
        SINGLETON_SESSION_MANAGEMENT_SERVICE.get().unwrap()
    }

    pub fn insert_session(&self, session: Session) {
        self.sessions.insert(session.session_id.clone(), session);
    }

    pub fn get_session(
        &self,
        session_id: &str,
    ) -> Option<dashmap::mapref::one::Ref<String, Session>> {
        self.sessions.get(session_id)
    }

    pub fn get_session_mut(
        &self,
        session_id: &str,
    ) -> Option<dashmap::mapref::one::RefMut<String, Session>> {
        self.sessions.get_mut(session_id)
    }

    pub fn remove_session(&self, session_id: &str) {
        self.sessions.remove(session_id);
    }
}
