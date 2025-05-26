use crate::proto::executor::ExecuteRequest;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::Request;

pub const SESSION_ID: &str = "session_id";
pub const ANONYMOUS: &str = "anonymous";

#[derive(Debug)]
pub enum SessionError {
    NotFound(String),
    InvalidLanguage(String),
    ExecutionError(String),
    Unauthenticated(String),
}

impl SessionError {
    pub fn message(&self) -> String {
        match self {
            SessionError::NotFound(id) => format!("Session with ID '{}' not found.", id),
            SessionError::InvalidLanguage(lang) => {
                format!("Invalid language specified: '{}'.", lang)
            }
            SessionError::ExecutionError(msg) => format!("Execution error: {}", msg),
            SessionError::Unauthenticated(msg) => format!("Unauthenticated: {}", msg),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionKey {
    pub session_id: String,
    pub language: String,
}

impl SessionKey {
    pub fn new(session_id: String, language: String) -> Self {
        SessionKey {
            session_id,
            language,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SessionValue {
    pub image: String,
}

impl SessionValue {
    pub fn new(image: String) -> Self {
        SessionValue { image }
    }
}

#[async_trait::async_trait]
pub trait SessionManagement {
    async fn add_session(
        &self,
        session_id: String,
        language: String,
        container_image: String,
    ) -> Result<(), SessionError>;

    async fn delete_session(&self, session_id: &str, language: &str) -> Result<(), SessionError>;

    async fn get_session_image(
        &self,
        session_id: &str,
        language: &str,
    ) -> Result<String, SessionError>;

    fn get_session_id(&self, request: &Request<ExecuteRequest>) -> Result<String, SessionError>;
}

#[derive(Clone, Debug, Default)]
pub struct SessionManagementService {
    sessions: Arc<Mutex<HashMap<SessionKey, SessionValue>>>,
}

impl SessionManagementService {
    pub fn new() -> Self {
        println!("Initializing SessionManagementService");
        SessionManagementService {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl SessionManagement for SessionManagementService {
    async fn add_session(
        &self,
        session_id: String,
        language: String,
        container_image: String,
    ) -> Result<(), SessionError> {

        let mut sessions = self.sessions.lock().await;
        let key = SessionKey::new(session_id.clone(), language.clone());

        if sessions.contains_key(&key) {
            return Err(SessionError::ExecutionError(format!(
                "Session already exists for ID '{}' and language '{}'",
                session_id, language
            )));
        }

        sessions.insert(key, SessionValue::new(container_image));
        Ok(())
    }

    async fn delete_session(&self, session_id: &str, language: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.lock().await;
        let key = SessionKey::new(session_id.to_string(), language.to_string());

        if sessions.remove(&key).is_none() {
            return Err(SessionError::NotFound(session_id.to_string()));
        }

        Ok(())
    }

    async fn get_session_image(
        &self,
        session_id: &str,
        language: &str,
    ) -> Result<String, SessionError> {
        let sessions = self.sessions.lock().await;
        let key = SessionKey::new(session_id.to_string(), language.to_string());

        match sessions.get(&key) {
            Some(val) => Ok(val.image.clone()),
            None => Err(SessionError::NotFound(session_id.to_string())),
        }
    }

    fn get_session_id(&self, request: &Request<ExecuteRequest>) -> Result<String, SessionError> {
        let session_id = request
            .metadata()
            .get(SESSION_ID)
            .and_then(|v: &tonic::metadata::MetadataValue<tonic::metadata::Ascii>| v.to_str().ok())
            .unwrap_or(ANONYMOUS)
            .to_string();

        if session_id == ANONYMOUS {
            return Err(SessionError::Unauthenticated(
                "Session ID is required for execution.".to_string(),
            ));
        }
        Ok(session_id)
    }
}
