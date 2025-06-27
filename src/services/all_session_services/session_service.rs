use crate::{
    models::{
        docker_models::DockerSupportedLanguage,
        in_memory_session_cache_model::{Session, SessionCache, SessionError, SessionErrorType},
    },
    services::{
        all_session_services::session_cache_service::SessionCache as session_cache_service,
        execution_services::code_editor_service::CodeEditor,
        websocket::websocket_message_service::{Code, Patch, WebSocketMessage},
    },
};

impl Session {
    pub fn new(
        session_id: String,
        language: DockerSupportedLanguage,
        initial_code: String,
    ) -> Self {
        Self {
            session_id,
            language,
            editor: CodeEditor::new(&initial_code),
        }
    }

    pub fn apply_full_code(&mut self, content: &str) {
        self.editor.update_from_string(content);
    }

    pub fn get_code(&self) -> String {
        self.editor.to_string()
    }
}

pub fn get_session_management_service() -> &'static SessionCache {
    session_cache_service::new()
}

pub fn update_create_session(message: &WebSocketMessage) -> Result<Session, SessionError> {
    let language_session_id = message.generate_session_id();
    if language_session_id.is_err() {
        return Err(SessionError::new(
            "Failed to generate session ID".to_string(),
            SessionErrorType::InvalidInput,
            400,
        ));
    }
    let session_management_service = get_session_management_service();
    let session = session_management_service
        .get_session_mut(language_session_id.as_ref().unwrap())
        .map(|mut mut_ref| {
            // Session exists
            match &message.code {
                Code::Patch { patches } => {
                    // Get existing code
                    let mut code_string = mut_ref.get_code();
                    // Apply each patch (you'll need to implement apply_patch)
                    for patch in patches {
                        code_string = apply_patch(&code_string, patch);
                    }
                    // Update session code
                    mut_ref.apply_full_code(&code_string);
                }
                Code::Full { content } => {
                    // Replace code with full content
                    mut_ref.apply_full_code(content);
                }
            }
            mut_ref.clone()
        });
    if session.is_none() {
        // Session does not exist, create new
        let language = message.get_language();
        if language.is_err() {
            return Err(SessionError::new(
                "Invalid language specified".to_string(),
                SessionErrorType::InvalidInput,
                400,
            ));
        }
        let new_session = Session::new(
            language_session_id.unwrap(),
            language.unwrap(),
            message.get_code_string(),
        );
        session_management_service.insert_session(new_session.clone());
        return Ok(new_session);
    }
    Ok(session.unwrap())
}

fn apply_patch(code: &str, patch: &Patch) -> String {
    // This is a naive implementation. You may need to handle multi-line and character positions more robustly.
    let mut lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();
    let Patch { start, end, text } = patch;
    if start.line == end.line {
        // Replace text in a single line
        if let Some(line) = lines.get_mut(start.line) {
            let before = &line[..start.ch];
            let after = &line[end.ch..];
            *line = format!("{}{}{}", before, text, after);
        }
    } else {
        // Multi-line patching (not implemented here)
        // You should implement this as per your requirements
    }
    lines.join("\n")
}

impl SessionErrorType {
    pub fn to_string(&self) -> String {
        match self {
            SessionErrorType::NotFound => "Session not found".to_string(),
            SessionErrorType::InvalidInput => "Invalid input provided".to_string(),
            SessionErrorType::InternalError => "Internal server error".to_string(),
        }
    }
}

impl SessionError {
    pub fn new(message: String, error_type: SessionErrorType, error_code: u16) -> Self {
        Self {
            message,
            error_type,
            error_code,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
