pub mod java_validator;
pub mod javascipt_validator;
pub mod python_validator;
pub mod validator;

use crate::models::docker_models::DockerSupportedLanguage as LanguageType;
use java_validator::JavaValidator;
use javascipt_validator::JavaScriptValidator;
use python_validator::PythonValidator;
use validator::SyntaxValidator;

pub fn get_validator(lang: LanguageType) -> Box<dyn SyntaxValidator> {
    match lang {
        LanguageType::Python => Box::new(PythonValidator),
        LanguageType::JavaScript => Box::new(JavaScriptValidator),
        LanguageType::Java => Box::new(JavaValidator),
    }
}
