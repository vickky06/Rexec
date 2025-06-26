use tree_sitter::Tree;
pub trait SyntaxValidator {
    fn validate(&self, code: &str) -> Result<Tree, String>;
}
