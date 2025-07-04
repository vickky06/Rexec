pub use crate::models::validation_models::ValidationError;
use tree_sitter::Tree;
pub trait SyntaxValidator {
    fn validate(&self, code: &str) -> Result<Tree, ValidationError>;
    fn validation_error(&self, error: &str) -> ValidationError {
        ValidationError::InvalidCode(error.to_string())
    }
}
