use super::validator::SyntaxValidator;
use tree_sitter::Parser;
use tree_sitter_javascript;

// unsafe extern "C" {
//     fn tree_sitter_javascript() -> Language;
// }

pub struct JavaScriptValidator;

impl SyntaxValidator for JavaScriptValidator {
    fn validate(&self, code: &str) -> Result<tree_sitter::Tree, String> {
        let language = tree_sitter_javascript::language();
        let mut parser = Parser::new();
        println!("{} code received", code);
        parser
            .set_language(language)
            .map_err(|e| format!("Set lang error: {:?}", e))?;

        let tree = parser
            .parse(code, None)
            .ok_or("Failed to parse Python code".to_string())?;

        let root_node = tree.root_node();
        if root_node.has_error() {
            let error_message = format!(
                "Syntax error detected in code at byte range {:?}",
                root_node.to_sexp()
            );
            return Err(error_message);
        }

        Ok(tree)
    }
}
