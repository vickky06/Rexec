use crate::models::code_editor_models::CodeEditorModel;

// Optionally, add a type alias if you want to refer to CodeEditorModel as CodeEditor
pub type CodeEditor = CodeEditorModel;

impl CodeEditorModel {
    pub fn new(code: &str) -> Self {
        let lines = code.lines().map(|s| s.to_string()).collect();
        Self { lines }
    }

    pub fn update_from_string(&mut self, content: &str) {
        self.lines = content.lines().map(String::from).collect();
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}
