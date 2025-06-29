pub fn sanitize_code_content(code: &str) -> String {
    code.chars()
        .filter(|&c| c == '\n' || c == '\r' || c == '\t' || c >= ' ')
        .collect()
}

pub fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
