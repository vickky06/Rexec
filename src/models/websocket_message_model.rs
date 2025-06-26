use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeType {
    Patch,
    Full,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub ch: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patch {
    pub start: Position,
    pub end: Position,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "code_type", rename_all = "lowercase")]
pub enum Code {
    Full { content: String },
    Patch { patches: Vec<Patch> },
}

// PRIMARY STRUCTURE FOR WEBSOCKET MESSAGES
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub session_id: String,
    pub language: String,
    #[serde(flatten)]
    pub code: Code,
}

/***
 * Example message (as JSON):
 * {
 *   "session_id": "abc123",
 *   "language": "rust",
 *   "code_type": "full",
 *   "content": "fn main() { println!(\"Hello, world!\"); }"
 * }
//
 * Or for a patch:
 * {
 *   "session_id": "abc123",
 *   "language": "rust",
 *   "code_type": "patch",
 *   "patches": [
 *     {
 *       "start": { "line": 1, "ch": 0 },
 *       "end": { "line": 1, "ch": 5 },
 *       "text": "let x = 42;"
 *     }
 *   ]
 * }
*/
