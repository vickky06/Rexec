pub fn sanitize_code_content(code: &str) -> String {
    code.chars()
        .filter(|&c| c == '\n' || c == '\r' || c == '\t' || c >= ' ')
        .collect()
}