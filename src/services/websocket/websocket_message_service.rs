use std::str::FromStr;

use crate::models::validation_models::ValidationError;
pub use crate::{
    models::websocket_message_model::{Code, CodeType, Patch, Position, WebSocketMessage},
    services::{
        helper_services::docker_service::DockerSupportedLanguage as DockerSupportedLanguageService,
        validation_services::request_validation::validation_service::ValidationError::InvalidLanguage,
    },
};

impl WebSocketMessage {
    // fn new(input: String) -> Self {
    //     let parsed: WebSocketMessage =
    //         serde_json::from_str(&input).expect("Failed to parse WebSocketMessage from input");
    //     parsed
    // }

    pub fn generate_session_id(&self) -> Result<String, ValidationError> {
        match self.get_language() {
            Ok(language) => {
                // Convert the language to a string representation
                let language_str = DockerSupportedLanguageService::to_string(&language);
                // Generate a session ID based on the language and content
                Ok(format!("{}-{}", language_str, self.session_id))
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_language(&self) -> Result<DockerSupportedLanguageService, ValidationError> {
        if DockerSupportedLanguageService::is_supported(&self.language).is_none() {
            return Err(InvalidLanguage((&self.language).clone()));
        }
        // Convert the language string to DockerSupportedLanguage enum
        Ok(DockerSupportedLanguageService::from_str(&self.language)
            .expect("Failed to parse DockerSupportedLanguage from language string"))
    }

    pub fn get_code_string(&self) -> String {
        // Convert the code to a string representation
        match &self.code {
            Code::Full { content } => content.clone(),
            Code::Patch { patches } => patches
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}

impl CodeType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "patch" => CodeType::Patch,
            "full" => CodeType::Full,
            _ => panic!("Unknown code type: {}", s),
        }
    }
}

impl Code {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "full" => Code::Full {
                content: String::new(),
            }, // Placeholder for full code
            "patch" => Code::Patch {
                patches: Vec::new(),
            }, // Placeholder for patches
            _ => panic!("Unknown code type: {}", s),
        }
    }
}

impl Position {
    fn new(line: usize, ch: usize) -> Self {
        Position { line, ch }
    }
}

impl Patch {
    fn new(start: Position, end: Position, text: String) -> Self {
        Patch { start, end, text }
    }
}
