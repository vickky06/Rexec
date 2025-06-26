use dashmap::DashMap;
use std::sync::Arc;

use crate::models::{code_editor_models::CodeEditorModel, docker_models::DockerSupportedLanguage};

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub language: DockerSupportedLanguage,
    pub editor: CodeEditorModel,
}
#[derive(Debug, Clone)]
pub enum SessionErrorType {
    NotFound,
    InvalidInput,
    InternalError,
}

pub struct SessionError {
    pub message: String,
    pub error_type: SessionErrorType,
    pub error_code: u16,
}

pub struct SessionCache {
    pub sessions: Arc<DashMap<String, Session>>,
}
