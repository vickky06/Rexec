pub struct ValidationService;

#[derive(Debug)]
pub enum ValidationError {
    InvalidLanguage(String),
    EmptyCode(),
    EmptyLanguage(),
    SessionIdError(String),
    InvalidCode(String),
    CodeTooLarge { actual: usize, max: usize },
}

pub struct ValidRequest {
    pub session_id: String,
    pub code: String,
    pub language: String,
}
